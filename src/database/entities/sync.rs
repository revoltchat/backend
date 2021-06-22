use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type UserSettings = HashMap<String, (i64, String)>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelCompositeKey {
    pub channel: String,
    pub user: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelUnread {
    #[serde(rename = "_id")]
    pub id: ChannelCompositeKey,

    pub last_id: Option<String>,
    pub mentions: Option<Vec<String>>,
}
