use crate::{database::*, notifications::events::ClientboundNotification, util::result::{Error, Result}};
use mongodb::bson::{DateTime, to_bson};
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

#[derive(Serialize, Deserialize, Debug)]
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
    pub async fn send(self) -> Result<()> {
        get_collection("messages")
            .insert_one(
                to_bson(&self).unwrap().as_document().unwrap().clone(),
                None
            )
            .await
            .map_err(|_| Error::DatabaseError { operation: "insert_one", with: "messages" })?;
        
        let channel = self.channel.clone();
        ClientboundNotification::Message(self)
            .publish(channel)
            .await
            .ok();

        Ok(())
    }
}
