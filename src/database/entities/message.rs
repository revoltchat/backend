use crate::{
    database::*,
    notifications::events::ClientboundNotification,
    util::result::{Error, Result},
};
use mongodb::bson::{to_bson, DateTime};
use serde::{Deserialize, Serialize};

/*#[derive(Serialize, Deserialize, Debug)]
pub struct PreviousEntry {
    pub content: String,
    pub time: DateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: String,
    pub nonce: Option<String>,
    pub channel: String,
    pub author: String,

    pub content: String,
    pub edited: Option<DateTime>,

    pub previous_content: Vec<PreviousEntry>,
}*/

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
    pub async fn publish(self) -> Result<()> {
        get_collection("messages")
            .insert_one(to_bson(&self).unwrap().as_document().unwrap().clone(), None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "insert_one",
                with: "messages",
            })?;

        let channel = self.channel.clone();
        ClientboundNotification::Message(self)
            .publish(channel)
            .await
            .ok();

        Ok(())
    }

    pub async fn publish_edit(self) -> Result<()> {
        let channel = self.channel.clone();
        ClientboundNotification::MessageEdit(self)
            .publish(channel)
            .await
            .ok();

        Ok(())
    }

    pub async fn publish_delete(self) -> Result<()> {
        let channel = self.channel.clone();
        ClientboundNotification::MessageDelete { id: self.id }
            .publish(channel)
            .await
            .ok();

        Ok(())
    }
}
