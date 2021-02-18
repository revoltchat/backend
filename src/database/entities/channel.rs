use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};
use mongodb::bson::{doc, from_document, to_document};
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
        last_message: Option<LastMessage>,
    },
}

impl Channel {
    pub fn id(&self) -> &str {
        match self {
            Channel::SavedMessages { id, .. }
            | Channel::DirectMessage { id, .. }
            | Channel::Group { id, .. } => id,
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
        ClientboundNotification::ChannelCreate(self)
            .publish(channel_id)
            .await
            .ok();

        Ok(())
    }

    pub async fn publish_update(&self, data: JsonValue) -> Result<()> {
        let id = self.id().to_string();
        ClientboundNotification::ChannelUpdate {
            id: id.clone(),
            data,
        }
        .publish(id)
        .await
        .ok();

        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        let id = self.id();
        get_collection("messages")
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

        ClientboundNotification::ChannelDelete { id: id.to_string() }
            .publish(id.to_string())
            .await
            .ok();

        Ok(())
    }
}
