use serde::{Deserialize, Serialize};

/// Composite primary key consisting of channel and user id
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct ChannelCompositeKey {
    /// Channel Id
    pub channel: String,
    /// User Id
    pub user: String,
}

/// Representation of the state of a channel from the perspective of a user
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct ChannelUnread {
    /// Composite key pointing to a user's view of a channel
    #[serde(rename = "_id")]
    pub id: ChannelCompositeKey,

    /// Id of the last message read in this channel by a user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    /// Array of message ids that mention the user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<String>>,
}
