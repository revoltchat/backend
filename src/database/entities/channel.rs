use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};
use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document, to_document, Document},
    options::FindOptions,
};
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LastMessage {
    #[serde(rename = "_id")]
    id: String,
    author: String,
    short: String,
}

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
        last_message: Option<LastMessage>,
    },
    Group {
        #[serde(rename = "_id")]
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        nonce: Option<String>,

        name: String,
        owner: String,
        description: String,
        recipients: Vec<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<File>,
        #[serde(skip_serializing_if = "Option::is_none")]
        last_message: Option<LastMessage>,
    },
    TextChannel {
        #[serde(rename = "_id")]
        id: String,
        server: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        nonce: Option<String>,

        name: String,
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<File>,
    }
}

impl Channel {
    pub fn id(&self) -> &str {
        match self {
            Channel::SavedMessages { id, .. }
            | Channel::DirectMessage { id, .. }
            | Channel::Group { id, .. }
            | Channel::TextChannel { id, .. } => id,
        }
    }

    pub async fn get(id: &str) -> Result<Channel> {
        let doc = get_collection("channels")
            .find_one(doc! { "_id": id }, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "channel",
            })?
            .ok_or_else(|| Error::UnknownChannel)?;

        from_document::<Channel>(doc).map_err(|_| Error::DatabaseError {
            operation: "from_document",
            with: "channel",
        })
    }

    pub async fn publish(self) -> Result<()> {
        get_collection("channels")
            .insert_one(
                to_document(&self).map_err(|_| Error::DatabaseError {
                    operation: "to_bson",
                    with: "channel",
                })?,
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "insert_one",
                with: "channel",
            })?;

        let channel_id = self.id().to_string();
        ClientboundNotification::ChannelCreate(self).publish(channel_id);

        Ok(())
    }

    pub async fn publish_update(&self, data: JsonValue) -> Result<()> {
        let id = self.id().to_string();
        ClientboundNotification::ChannelUpdate {
            id: id.clone(),
            data,
            clear: None,
        }
        .publish(id);

        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        let id = self.id();
        let messages = get_collection("messages");

        // Check if there are any attachments we need to delete.
        let message_ids = messages
            .find(
                doc! {
                    "channel": id,
                    "attachment": {
                        "$exists": 1
                    }
                },
                FindOptions::builder().projection(doc! { "_id": 1 }).build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "fetch_many",
                with: "messages",
            })?
            .filter_map(async move |s| s.ok())
            .collect::<Vec<Document>>()
            .await
            .into_iter()
            .filter_map(|x| x.get_str("_id").ok().map(|x| x.to_string()))
            .collect::<Vec<String>>();

        // If we found any, mark them as deleted.
        if message_ids.len() > 0 {
            get_collection("attachments")
                .update_many(
                    doc! {
                        "message_id": {
                            "$in": message_ids
                        }
                    },
                    doc! {
                        "$set": {
                            "deleted": true
                        }
                    },
                    None,
                )
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "update_many",
                    with: "attachments",
                })?;
        }

        // And then delete said messages.
        messages
            .delete_many(
                doc! {
                    "channel": id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_many",
                with: "messages",
            })?;

        get_collection("channels")
            .delete_one(
                doc! {
                    "_id": id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_one",
                with: "channel",
            })?;

        ClientboundNotification::ChannelDelete { id: id.to_string() }.publish(id.to_string());

        if let Channel::Group { icon, .. } = self {
            if let Some(attachment) = icon {
                attachment.delete().await?;
            }
        }

        Ok(())
    }
}
