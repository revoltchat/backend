use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LastMessage {
    id: String,
    user_id: String,
    short_content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Channel {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "type")]
    pub channel_type: u8,

    // DM: whether the DM is active
    pub active: Option<bool>,
    // DM + GDM: last message in channel
    pub last_message: Option<LastMessage>,
    // DM + GDM: recipients for channel
    pub recipients: Option<Vec<String>>,
    // GDM: owner of group
    pub owner: Option<String>,
    // GUILD: channel parent
    pub guild: Option<String>,
    // GUILD + GDM: channel name
    pub name: Option<String>,
    // GUILD + GDM: channel description
    pub description: Option<String>,
}
