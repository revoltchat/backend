use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::{doc, from_document};
use rocket::serde::json::Value;
use futures::StreamExt;

#[get("/<target>/pins")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
    let target = target.fetch_channel().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel()
        .await?;

    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    let mut cursor = get_collection("messages").find(doc! {"pinned": true, "channel": target.id()}, None)
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "fetch_many",
            with: "message",
    })?;

    let mut messages: Vec<Message> = vec![];

    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            let message: Message = from_document(doc).map_err(|_| Error::DatabaseError {
                operation: "from_document",
                with: "message",
            })?;
            messages.push(message);
        }
    }

    Ok(json!(messages))
}
