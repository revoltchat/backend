use crate::database::*;
use crate::util::result::{Error, Result};

use futures::StreamExt;
use mongodb::bson::{doc, Document};
use mongodb::options::FindOptions;
use rocket::serde::json::Value;

#[get("/<target>/mutual")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
    let users = get_collection("users")
        .find(
            doc! {
                "$and": [
                    { "relations": { "$elemMatch": { "_id": &user.id, "status": "Friend" } } },
                    { "relations": { "$elemMatch": { "_id": &target.id, "status": "Friend" } } }
                ]
            },
            FindOptions::builder().projection(doc! { "_id": 1 }).build(),
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "users",
        })?
        .filter_map(async move |s| s.ok())
        .collect::<Vec<Document>>()
        .await
        .into_iter()
        .filter_map(|x| x.get_str("_id").ok().map(|x| x.to_string()))
        .collect::<Vec<String>>();

    Ok(json!({ "users": users }))
}
