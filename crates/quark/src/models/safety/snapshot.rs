use serde::{Deserialize, Serialize};

use crate::models::{Message, Server, User};

/// Enum to map into different models
/// that can be saved in a snapshot
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(tag = "_type")]
pub enum SnapshotContent {
    Message(Message),
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
