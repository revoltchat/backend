use crate::database::*;
use crate::util::result::{Error, Result};

use futures::StreamExt;
use mongodb::bson::{Document, doc, from_document};
use serde_json::Value;

#[get("/@me")]
pub async fn fetch_owned_bots(user: User) -> Result<Value> {
    let bots = get_collection("bots")
        .find(
            doc! {
                "owner": &user.id
            },
            None
        )
        .await
        .map_err(|_| Error::DatabaseError {
            with: "bots",
            operation: "find"
        })?
        .filter_map(async move |s| s.ok())
        .collect::<Vec<Document>>()
        .await
        .into_iter()
        .filter_map(|x| from_document(x).ok())
        .collect::<Vec<Bot>>();
    
    let users = get_collection("users")
        .find(
            doc! {
                "bot.owner": &user.id
            },
            None
        )
        .await
        .map_err(|_| Error::DatabaseError {
            with: "users",
            operation: "find"
        })?
        .filter_map(async move |s| s.ok())
        .collect::<Vec<Document>>()
        .await
        .into_iter()
        .filter_map(|x| from_document(x).ok())
        .collect::<Vec<User>>();
    
    Ok(json!({
        "bots": bots,
        "users": users
    }))
}
