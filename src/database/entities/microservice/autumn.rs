use mongodb::bson::{doc, from_document};
use serde::{Deserialize, Serialize};

use crate::database::*;
use crate::util::result::{Error, Result};
use crate::util::variables::AUTUMN_URL;

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
    deleted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reported: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    server_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    object_id: Option<String>,
}

impl File {
    pub async fn find_and_use(
        attachment_id: &str,
        tag: &str,
        parent_type: &str,
        parent_id: &str,
    ) -> Result<File> {
        if let Ok(attachment) = db_conn().get_attachment(attachment_id, tag, parent_type).await {
            db_conn().link_attachment_to_parent(&attachment.id, parent_type, parent_id).await?;
            Ok(attachment)
        } else {
            Err(Error::UnknownAttachment)
        }
    }

    pub async fn delete(&self) -> Result<()> {
        db_conn().delete_attachment(&self.id).await
    }

    pub fn get_autumn_url(&self) -> String {
        format!("{}/{}/{}", AUTUMN_URL.as_str(), self.tag, self.id)
    }
}
