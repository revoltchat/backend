use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum Metadata {
    File,
    Text,
    Image { width: isize, height: isize },
    Video { width: isize, height: isize },
    Audio,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
    #[serde(rename = "_id")]
    pub id: String,
    tag: String,
    filename: String,
    metadata: Metadata,
    content_type: String,
    size: isize,

    #[serde(skip_serializing_if = "Option::is_none")]
    message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    server_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    object_id: Option<String>,
}
