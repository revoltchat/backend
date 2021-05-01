use mongodb::bson::{doc, from_document};
use serde::{Deserialize, Serialize};

use crate::util::result::{Error, Result};
use crate::database::*;

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
    message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    server_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    object_id: Option<String>,
}

impl File {
    pub async fn find_and_use(attachment_id: &str, tag: &str, parent_type: &str, parent_id: &str) -> Result<File> {
        let attachments = get_collection("attachments");
        let key = format!("{}_id", parent_type);
        if let Some(doc) = attachments
            .find_one(
                doc! {
                    "_id": attachment_id,
                    "tag": &tag,
                    key.clone(): {
                        "$exists": false
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "attachment",
            })?
        {
            let attachment = from_document::<File>(doc).map_err(|_| Error::DatabaseError {
                operation: "from_document",
                with: "attachment",
            })?;

            attachments
                .update_one(
                    doc! {
                        "_id": &attachment.id
                    },
                    doc! {
                        "$set": {
                            key: &parent_id
                        }
                    },
                    None,
                )
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "update_one",
                    with: "attachment",
                })?;

            Ok(attachment)
        } else {
            Err(Error::UnknownAttachment)
        }
    }
}
