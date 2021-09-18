use std::collections::HashMap;

use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};
use crate::util::variables::MAX_GROUP_SIZE;
use futures::StreamExt;
use mongodb::bson::Bson;
use mongodb::{
    bson::{doc, to_document, Document},
    options::FindOptions,
};
use rocket::serde::json::Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "channel_type")]
pub enum Channel {
    SavedMessages {
        #[serde(rename = "_id")]
        id: String,
        user: String,
    },
    DirectMessage {
        #[serde(rename = "_id")]
        id: String,

        active: bool,
        recipients: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        last_message_id: Option<String>,
    },
    Group {
        #[serde(rename = "_id")]
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        nonce: Option<String>,

        name: String,
        owner: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        recipients: Vec<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<File>,
        #[serde(skip_serializing_if = "Option::is_none")]
        last_message_id: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        permissions: Option<i32>,

        #[serde(skip_serializing_if = "entities::server::if_false", default)]
        nsfw: bool
   },
    TextChannel {
        #[serde(rename = "_id")]
        id: String,
        server: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        nonce: Option<String>,

        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<File>,
        #[serde(skip_serializing_if = "Option::is_none")]
        last_message_id: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        default_permissions: Option<i32>,
        #[serde(default = "HashMap::new", skip_serializing_if = "HashMap::is_empty")]
        role_permissions: HashMap<String, i32>,
        
        #[serde(skip_serializing_if = "entities::server::if_false", default)]
        nsfw: bool
    },
    VoiceChannel {
        #[serde(rename = "_id")]
        id: String,
        server: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        nonce: Option<String>,

        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<File>,

        #[serde(skip_serializing_if = "Option::is_none")]
        default_permissions: Option<i32>,
        #[serde(default = "HashMap::new", skip_serializing_if = "HashMap::is_empty")]
        role_permissions: HashMap<String, i32>,

        #[serde(skip_serializing_if = "entities::server::if_false", default)]
        nsfw: bool
    },
}

impl Channel {
    pub fn id(&self) -> &str {
        match self {
            Channel::SavedMessages { id, .. }
            | Channel::DirectMessage { id, .. }
            | Channel::Group { id, .. }
            | Channel::TextChannel { id, .. }
            | Channel::VoiceChannel { id, .. } => id,
        }
    }
    pub fn has_messaging(&self) -> Result<()> {
        match self {
            Channel::SavedMessages { .. }
            | Channel::DirectMessage { .. }
            | Channel::Group { .. }
            | Channel::TextChannel { .. } => Ok(()),
            Channel::VoiceChannel { .. } => Err(Error::InvalidOperation)
        }
    }

    pub async fn publish(self) -> Result<()> {
        db_conn().add_channel(&self).await?;
        let channel_id = self.id().to_string();
        ClientboundNotification::ChannelCreate(self).publish(channel_id);

        Ok(())
    }

    pub async fn publish_update(&self, data: Value) -> Result<()> {
        let id = self.id().to_string();
        ClientboundNotification::ChannelUpdate {
            id: id.clone(),
            data,
            clear: None,
        }
        .publish(id);

        Ok(())
    }

    pub async fn delete_messages(channel_ids: &Vec<String>) -> Result<()> {

        // Delete any unreads.
        db_conn().delete_channel_unreads(channel_ids).await?;

        // Check if there are any attachments we need to delete.
        let message_ids = db_conn().get_ids_from_messages_with_attachments(channel_ids).await?;

        // If we found any, mark them as deleted.
        if message_ids.len() > 0 {
            db_conn().delete_attachments_of_messages(&message_ids).await?;
        }

        // And then delete said messages.
        db_conn().delete_messages_from_channels(channel_ids).await
    }

    pub async fn delete(&self) -> Result<()> {
        let id = self.id();

        // Delete any invites.
        db_conn().delete_invites_associated_to_channel(id).await?;

        // Delete messages.
        match &self {
            Channel::VoiceChannel { .. } => {},
            _ => {
                Channel::delete_messages(&vec![id.to_string()]).await?;
            }
        }

        // Remove from server object.
        match &self {
            Channel::TextChannel { server, .. }
            | Channel::VoiceChannel { server, .. } => {
                let server = Ref::from_unchecked(server.clone()).fetch_server().await?;
                let mut update = doc! {
                    "$pull": {
                        "channels": id
                    }
                };

                if let Some(sys) = &server.system_messages {
                    let mut unset = doc! {};

                    if let Some(cid) = &sys.user_joined {
                        if id == cid {
                            unset.insert("system_messages.user_joined", 1);
                        }
                    }

                    if let Some(cid) = &sys.user_left {
                        if id == cid {
                            unset.insert("system_messages.user_left", 1);
                        }
                    }

                    if let Some(cid) = &sys.user_kicked {
                        if id == cid {
                            unset.insert("system_messages.user_kicked", 1);
                        }
                    }

                    if let Some(cid) = &sys.user_banned {
                        if id == cid {
                            unset.insert("system_messages.user_banned", 1);
                        }
                    }

                    if unset.len() > 0 {
                        update.insert("$unset", unset);
                    }
                }
                db_conn().apply_server_changes(&server.id, update).await?;
            },
            _ => {}
        }

        // Finally, delete the channel object.
        db_conn().delete_channel(id).await?;

        ClientboundNotification::ChannelDelete { id: id.to_string() }.publish(id.to_string());

        if let Channel::Group { icon, .. } = self {
            if let Some(attachment) = icon {
                attachment.delete().await?;
            }
        }

        Ok(())
    }

    pub async fn add_to_group(&self, member: String, by_user: String) -> Result<()> {
        if let Channel::Group { id, recipients, .. } = &self {
            if recipients.len() >= *MAX_GROUP_SIZE {
                Err(Error::GroupTooLarge {
                    max: *MAX_GROUP_SIZE,
                })?
            }

            if recipients.iter().find(|x| *x == &member).is_some() {
                Err(Error::AlreadyInGroup)?
            }
            db_conn().add_recipient_to_channel(&id, &member).await?;

            ClientboundNotification::ChannelGroupJoin {
                id: id.clone(),
                user: member.clone(),
            }
            .publish(id.clone());

            Content::SystemMessage(SystemMessage::UserAdded {
                id: member,
                by: by_user,
            })
            .send_as_system(&self)
            .await
            .ok();
            Ok(())
        } else {
            Err(Error::InvalidOperation)
        }
    }
}
