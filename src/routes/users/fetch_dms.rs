use crate::database::*;
use crate::util::result::{Error, Result};

use futures::StreamExt;
use mongodb::bson::doc;
use rocket_contrib::json::JsonValue;

#[get("/dms")]
pub async fn req(user: User) -> Result<JsonValue> {
    let mut cursor = get_collection("channels")
        .find(
            doc! {
                "$or": [
                    {
                        "type": "DirectMessage",
                        "active": true
                    },
                    {
                        "type": "Group"
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
