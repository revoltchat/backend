use crate::database::entities::{Channel, User};
use mongodb::bson::{Bson, doc, from_bson};
use crate::util::result::{Error, Result};
use crate::database::get_collection;
use rocket_contrib::json::JsonValue;
use futures::StreamExt;

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
            None
        )
        .await
        .map_err(|_| Error::DatabaseError { operation: "find", with: "channels" })?;
    
    let mut channels: Vec<Channel> = vec![];
    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            channels.push(
                from_bson(Bson::Document(doc))
                    .map_err(|_| Error::DatabaseError { operation: "from_bson", with: "channel" })?
            );
        }
    }

    Ok(json!(
        channels
    ))
}
