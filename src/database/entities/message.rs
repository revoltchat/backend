use crate::{
    database::*,
    notifications::events::ClientboundNotification,
    util::result::{Error, Result},
};
use mongodb::bson::{doc, to_bson, DateTime};
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    pub channel: String,
    pub author: String,

    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited: Option<DateTime>,
}

impl Message {
    pub fn create(author: String, channel: String, content: String) -> Message {
        Message {
            id: Ulid::new().to_string(),
            nonce: None,
            channel,
            author,

            content,
            edited: None,
        }
    }

    pub async fn publish(self, channel: &Channel) -> Result<()> {
        get_collection("messages")
            .insert_one(to_bson(&self).unwrap().as_document().unwrap().clone(), None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "insert_one",
                with: "message",
            })?;

        // ! temp code
        let channels = get_collection("channels");
        match channel {
            Channel::DirectMessage { id, .. } => {
                channels
                    .update_one(
                        doc! { "_id": id },
                        doc! {
                            "$set": {
                                "active": true,
                                "last_message": {
                                    "_id": self.id.clone(),
                                    "author": self.author.clone(),
                                    "short": self.content.chars().take(24).collect::<String>()
                                }
                            }
                        },
                        None,
                    )
                    .await
                    .map_err(|_| Error::DatabaseError {
                        operation: "update_one",
                        with: "channel",
                    })?;
            }
            Channel::Group { id, .. } => {
                channels
                    .update_one(
                        doc! { "_id": id },
                        doc! {
                            "$set": {
                                "last_message": {
                                    "_id": self.id.clone(),
                                    "author": self.author.clone(),
                                    "short": self.content.chars().take(24).collect::<String>()
                                }
                            }
                        },
                        None,
                    )
                    .await
                    .map_err(|_| Error::DatabaseError {
                        operation: "update_one",
                        with: "channel",
                    })?;
            }
            _ => {}
        }

        ClientboundNotification::Message(self)
            .publish(channel.id().to_string())
            .await
            .ok();

        Ok(())
    }

    pub async fn publish_update(&self, data: JsonValue) -> Result<()> {
        let channel = self.channel.clone();
        ClientboundNotification::MessageUpdate {
            id: self.id.clone(),
            data,
        }
        .publish(channel)
        .await
        .ok();

        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        get_collection("messages")
            .delete_one(
                doc! {
                    "_id": &self.id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_one",
                with: "message",
            })?;

        let channel = self.channel.clone();
        ClientboundNotification::MessageDelete {
            id: self.id.clone(),
        }
        .publish(channel)
        .await
        .ok();

        Ok(())
    }
}
