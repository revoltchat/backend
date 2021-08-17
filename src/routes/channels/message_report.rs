use crate::database::*;
use crate::util::result::{Error, Result};

use chrono::Utc;
use mongodb::bson::{doc, Bson, DateTime, Document};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 2000))]
    reason: String,
    comments: String,
}

#[patch("/<target>/messages/<msg>/report", data = "<edit>")]
pub async fn req(user: User, target: Ref, msg: Ref, data: Json<Data>) -> Result<()> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let channel = target.fetch_channel().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&channel)
        .for_channel()
        .await?;
    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    let mut message = msg.fetch_message(&channel).await?;
    if message.author == user.id {
        Err(Error::CannotSelfReport)?
    }

    get_collection("message_reports")
        .insert_one(
            doc! {
                "_id": Ulid::new().to_string(),
                "message": message,
                "reason": data.reason,
                "comments": data.comments
            }
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "insert_one",
            with: "message_report",
        })?;

    return Ok(());
}
