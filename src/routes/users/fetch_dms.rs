use crate::database::*;
use crate::util::result::{Error, Result};

use futures::StreamExt;
use mongodb::bson::doc;
use rocket::serde::json::Value;

#[get("/dms")]
pub async fn req(user: User) -> Result<Value> {
    let mut cursor = get_collection("channels")
        .find(
            doc! {
                "$or": [
                    {
                        "channel_type": "DirectMessage",
                        "active": true
                    },
                    {
                        "channel_type": "Group"
                    }
                ],
                "recipients": user.id
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "channels",
        })?;

    let mut channels = vec![];
    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            channels.push(doc);
        }
    }

    Ok(json!(channels))
}
