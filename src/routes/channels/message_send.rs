use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::{
    bson::{doc, from_document},
    options::FindOneOptions,
};
use rocket_contrib::json::{Json, JsonValue};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 0, max = 2000))]
    content: String,
    // Maximum length of 36 allows both ULIDs and UUIDs.
    #[validate(length(min = 1, max = 36))]
    nonce: String,
    #[validate(length(min = 1, max = 128))]
    attachment: Option<String>,
}

#[post("/<target>/messages", data = "<message>")]
pub async fn req(user: User, target: Ref, message: Json<Data>) -> Result<JsonValue> {
    message
        .validate()
        .map_err(|error| Error::FailedValidation { error })?;
    
    if message.content.len() == 0 && message.attachment.is_none() {
        return Err(Error::EmptyMessage);
    }

    let target = target.fetch_channel().await?;
    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel()
        .await?;

    if !perm.get_send_message() {
        Err(Error::MissingPermission)?
    }

    if get_collection("messages")
        .find_one(
            doc! {
                "nonce": &message.nonce
            },
            FindOneOptions::builder()
                .projection(doc! { "_id": 1 })
                .build(),
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find_one",
            with: "message",
        })?
        .is_some()
    {
        Err(Error::DuplicateNonce)?
    }

    let id = Ulid::new().to_string();
    let attachments = get_collection("attachments");
    let attachment = if let Some(attachment_id) = &message.attachment {
        if let Some(doc) = attachments
            .find_one(
                doc! {
                    "_id": attachment_id,
                    "message_id": {
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
                            "message_id": &id
                        }
                    },
                    None,
                )
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "update_one",
                    with: "attachment",
                })?;

            Some(attachment)
        } else {
            return Err(Error::UnknownAttachment);
        }
    } else {
        None
    };

    let msg = Message {
        id,
        channel: target.id().to_string(),
        author: user.id,

        content: Content::Text(message.content.clone()),
        attachment,
        nonce: Some(message.nonce.clone()),
        edited: None,
        embeds: None
    };

    msg.clone().publish(&target).await?;

    Ok(json!(msg))
}
