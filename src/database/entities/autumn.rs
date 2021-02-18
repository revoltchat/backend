use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum Metadata {
    File,
    Image { width: isize, height: isize },
    Video { width: isize, height: isize },
    Audio,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
    #[serde(rename = "_id")]
    pub id: String,
    filename: String,
    metadata: Metadata,
    content_type: String,

    message_id: Option<String>,
}
