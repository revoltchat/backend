use serde::{Deserialize, Serialize};

use crate::models::{Channel, Message, Server, User};

/// Enum to map into different models
/// that can be saved in a snapshot
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(tag = "_type")]
pub enum SnapshotContent {
    Message {
        /// Context before the message
        #[serde(rename = "_prior_context", default)]
        prior_context: Vec<Message>,

        /// Context after the message
        #[serde(rename = "_leading_context", default)]
        leading_context: Vec<Message>,

        /// Message
        #[serde(flatten)]
        message: Message,
    },
    Server(Server),
    User(User),
}

/// Snapshot of some content
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Snapshot {
    /// Unique Id
    #[serde(rename = "_id")]
    pub id: String,
    /// Report parent Id
    pub report_id: String,
    /// Snapshot of content
    pub content: SnapshotContent,
}

/// Snapshot of some content with required data to render
#[derive(Serialize, JsonSchema, Debug)]
pub struct SnapshotWithContext {
    /// Snapshot itself
    #[serde(flatten)]
    pub snapshot: Snapshot,
    /// Users involved in snapshot
    #[serde(rename = "_users", skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<User>,
    /// Channels involved in snapshot
    #[serde(rename = "_channels", skip_serializing_if = "Vec::is_empty")]
    pub channels: Vec<Channel>,
    /// Server involved in snapshot
    #[serde(rename = "_server", skip_serializing_if = "Option::is_none")]
    pub server: Option<Server>,
}
