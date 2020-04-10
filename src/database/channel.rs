use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LastMessage {
    id: String,
    user_id: String,
    short_content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Channel {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "type")]
    pub channel_type: u8,

    // for Direct Messages
    pub active: Option<bool>,

    // for DMs / GDMs
    pub last_message: Option<LastMessage>,
    pub recipients: Option<Vec<String>>,

    // for GDMs
    pub owner: Option<String>,

    // for Guilds
    pub guild: Option<String>,

    // for Guilds and Group DMs
    pub name: Option<String>,
    pub description: Option<String>,
}
