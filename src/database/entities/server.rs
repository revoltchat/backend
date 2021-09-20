use std::collections::HashMap;

use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};
use futures::StreamExt;
use mongodb::bson::{Bson, doc};
use mongodb::bson::from_document;
use mongodb::bson::to_document;
use mongodb::bson::Document;
use rocket::serde::json::Value;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemberCompositeKey {
    pub server: String,
    pub user: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Member {
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<File>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>
}

pub type PermissionTuple = (
    i32, // server permission
    i32  // channel permission
);

pub fn if_false(t: &bool) -> bool {
    !t
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Role {
    pub name: String,
    pub permissions: PermissionTuple,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,
    #[serde(skip_serializing_if = "if_false", default)]
    pub hoist: bool,
    #[serde(default)]
    pub rank: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Category {
    pub id: String,
    pub title: String,
    pub channels: Vec<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ban {
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,
    pub reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemMessageChannels {
    pub user_joined: Option<String>,
    pub user_left: Option<String>,
    pub user_kicked: Option<String>,
    pub user_banned: Option<String>,
}

pub enum RemoveMember {
    Leave,
    Kick,
    Ban,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Server {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    pub owner: String,

    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub channels: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<Category>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_messages: Option<SystemMessageChannels>,

    #[serde(default = "HashMap::new", skip_serializing_if = "HashMap::is_empty")]
    pub roles: HashMap<String, Role>,
    pub default_permissions: PermissionTuple,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<File>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<File>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<i32>,

    #[serde(skip_serializing_if = "if_false", default)]
    pub nsfw: bool
}

impl Server {
    pub async fn create(self) -> Result<()> {
        db_conn().add_server(&self).await
    }

    pub async fn publish_update(&self, data: Value) -> Result<()> {
        ClientboundNotification::ServerUpdate {
            id: self.id.clone(),
            data,
            clear: None,
        }
        .publish(self.id.clone());

        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        // Check if there are any attachments we need to delete.
        Channel::delete_messages(&self.channels).await?;

        // Delete all channels.
        db_conn().delete_all_channels_from_server(&self.id).await?;

        // Delete any associated objects, e.g. unreads and invites.
        db_conn().delete_invites_associated_to_channels(&self.channels).await?;

        // Delete members and bans.
        db_conn().delete_bans_of_server(&self.id).await?;
        db_conn().delete_members_of_server(&self.id).await?;

        // Delete server icon / banner.
        if let Some(attachment) = &self.icon {
            attachment.delete().await?;
        }

        if let Some(attachment) = &self.banner {
            attachment.delete().await?;
        }

        // Delete the server
        db_conn().delete_server(&self.id).await?;

        ClientboundNotification::ServerDelete {
            id: self.id.clone(),
        }
        .publish(self.id.clone());

        Ok(())
    }

    pub async fn fetch_members(id: &str) -> Result<Vec<Member>> {
        db_conn().get_server_members(id).await
    }

    pub async fn fetch_member_ids(id: &str) -> Result<Vec<String>> {
        Ok(db_conn().get_server_members(id).await?.iter().map(|e| e.id.user.to_string()).collect())
    }

    pub async fn mark_as_read(&self, id: &str) -> Result<()> {
        let current_time = Ulid::new().to_string();
        db_conn().delete_multi_channel_unreads_for_user(&self.channels, &id).await?;
        db_conn().add_channels_to_unreads_for_user(&self.channels, id, &current_time).await
    }

    pub async fn join_member(&self, id: &str) -> Result<()> {
        // Check if user is banned.
        if db_conn().is_user_banned(&self.id, id).await? {
            return Err(Error::Banned);
        }

        // Add user to server.
        db_conn().add_server_member(&self.id, id).await?;

        // Announce that user joined server.
        ClientboundNotification::ServerMemberJoin {
            id: self.id.clone(),
            user: id.to_string(),
        }
        .publish(self.id.clone());

        // Broadcast join message.
        if let Some(channels) = &self.system_messages {
            if let Some(cid) = &channels.user_joined {
                let channel = Ref::from_unchecked(cid.clone()).fetch_channel().await?;
                Content::SystemMessage(SystemMessage::UserJoined { id: id.to_string() })
                    .send_as_system(&channel)
                    .await?;
            }
        }

        // Mark entire server as read.
        self.mark_as_read(&id).await?;

        Ok(())
    }

    pub async fn remove_member(&self, id: &str, removal: RemoveMember) -> Result<()> {
        let delete_count = db_conn().delete_server_member(&self.id, id).await?;
        if delete_count > 0 {
            ClientboundNotification::ServerMemberLeave {
                id: self.id.clone(),
                user: id.to_string(),
            }
            .publish(self.id.clone());

            if let Some(channels) = &self.system_messages {
                let message = match removal {
                    RemoveMember::Leave => {
                        if let Some(cid) = &channels.user_left {
                            Some((cid.clone(), SystemMessage::UserLeft { id: id.to_string() }))
                        } else {
                            None
                        }
                    }
                    RemoveMember::Kick => {
                        if let Some(cid) = &channels.user_kicked {
                            Some((
                                cid.clone(),
                                SystemMessage::UserKicked { id: id.to_string() },
                            ))
                        } else {
                            None
                        }
                    }
                    RemoveMember::Ban => {
                        if let Some(cid) = &channels.user_banned {
                            Some((
                                cid.clone(),
                                SystemMessage::UserBanned { id: id.to_string() },
                            ))
                        } else {
                            None
                        }
                    }
                };

                if let Some((cid, message)) = message {
                    let channel = Ref::from_unchecked(cid).fetch_channel().await?;
                    Content::SystemMessage(message)
                        .send_as_system(&channel)
                        .await?;
                }
            }
        }

        Ok(())
    }

    pub async fn get_member_count(id: &str) -> Result<i64> {
        db_conn().get_server_member_count(id).await
    }
}
