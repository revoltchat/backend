use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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
}
