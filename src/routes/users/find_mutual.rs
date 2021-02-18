use crate::database::*;
use crate::util::result::{Error, Result};

use futures::StreamExt;
use mongodb::bson::{doc, Document};
use mongodb::options::FindOptions;
use rocket_contrib::json::JsonValue;

#[get("/<target>/mutual")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let channels = get_collection("channels")
        .find(
            doc! {
                "$or": [
                    { "type": "Group" },
                ],
                "$and": [
                    { "recipients": &user.id },
                    { "recipients": &target.id }
                ]
            },
            FindOptions::builder().projection(doc! { "_id": 1 }).build(),
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "channels",
        })?
        .filter_map(async move |s| s.ok())
        .collect::<Vec<Document>>()
        .await
        .into_iter()
        .filter_map(|x| x.get_str("_id").ok().map(|x| x.to_string()))
        .collect::<Vec<String>>();

    Ok(json!({ "channels": channels }))
}
