use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;
use rocket_contrib::json::{Json, JsonValue};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 2000))]
    content: String,
    // Maximum length of 36 allows both ULIDs and UUIDs.
    #[validate(length(min = 1, max = 36))]
    nonce: String,
}

#[post("/<target>/messages", data = "<message>")]
pub async fn req(user: User, target: Ref, message: Json<Data>) -> Result<JsonValue> {
    message
        .validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let target = target.fetch_channel().await?;

    let perm = permissions::channel::calculate(&user, &target).await;
    if !perm.get_send_message() {
        Err(Error::LabelMe)?
    }

    if get_collection("messages")
        .find_one(
            doc! {
                "nonce": &message.nonce
            },
            None,
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

    let msg = Message {
        id: Ulid::new().to_string(),
        channel: target.id().to_string(),
        author: user.id,

        content: message.content.clone(),
        nonce: Some(message.nonce.clone()),
        edited: None,
    };

    msg.clone().publish().await?;

    Ok(json!(msg))
}
