use crate::database::*;
use crate::util::result::{Error, Result};

use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document},
    options::FindOptions,
};
use rocket::request::Form;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, FromForm)]
pub struct Options {
    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,
    #[validate(length(min = 26, max = 26))]
    before: Option<String>,
    #[validate(length(min = 26, max = 26))]
    after: Option<String>,
}

#[get("/<target>/messages?<options..>")]
pub async fn req(user: User, target: Ref, options: Form<Options>) -> Result<JsonValue> {
    options
        .validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let target = target.fetch_channel().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel().await?;
    if !perm.get_view() {
        Err(Error::LabelMe)?
    }

    let mut query = doc! { "channel": target.id() };

    if let Some(before) = &options.before {
        query.insert("_id", doc! { "$lt": before });
    }

    if let Some(after) = &options.after {
        query.insert("_id", doc! { "$gt": after });
    }

    let mut cursor = get_collection("messages")
        .find(
            query,
            FindOptions::builder()
                .limit(options.limit.unwrap_or(50))
                .sort(doc! {
                    "_id": -1
                })
                .build(),
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "messages",
        })?;

    let mut messages = vec![];
    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            messages.push(
                from_document::<Message>(doc).map_err(|_| Error::DatabaseError {
                    operation: "from_document",
                    with: "message",
                })?,
            );
        }
    }

    Ok(json!(messages))
}
