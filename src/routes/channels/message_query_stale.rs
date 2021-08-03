use crate::database::*;
use crate::util::result::{Error, Result};

use futures::StreamExt;
use mongodb::bson::{doc, from_document};
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Options {
    ids: Vec<String>,
}

#[post("/<target>/messages/stale", data = "<data>")]
pub async fn req(user: User, target: Ref, data: Json<Options>) -> Result<Value> {
    if data.ids.len() > 150 {
        return Err(Error::TooManyIds);
    }

    let target = target.fetch_channel().await?;
    target.has_messaging()?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel()
        .await?;
    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    let mut cursor = get_collection("messages")
        .find(
            doc! {
                "_id": {
                    "$in": &data.ids
                },
                "channel": target.id()
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "messages",
        })?;

    let mut updated = vec![];
    let mut found_ids = vec![];
    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            let msg = from_document::<Message>(doc).map_err(|_| Error::DatabaseError {
                operation: "from_document",
                with: "message",
            })?;

            found_ids.push(msg.id.clone());
            if msg.edited.is_some() {
                updated.push(msg);
            }
        }
    }

    let mut deleted = vec![];
    for id in &data.ids {
        if found_ids.iter().find(|x| *x == id).is_none() {
            deleted.push(id);
        }
    }

    Ok(json!({
        "updated": updated,
        "deleted": deleted
    }))
}
