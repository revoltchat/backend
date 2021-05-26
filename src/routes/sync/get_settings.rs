use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use mongodb::options::FindOneOptions;
use rocket_contrib::json::{Json, JsonValue};

#[derive(Serialize, Deserialize)]
pub struct Options {
    keys: Vec<String>
}

#[post("/settings/fetch", data = "<options>")]
pub async fn req(user: User, options: Json<Options>) -> Result<JsonValue> {
    let options = options.into_inner();
    let mut projection = doc! {
        "_id": 0,
    };

    for key in options.keys {
        projection.insert(key, 1);
    }

    if let Some(doc) = get_collection("user_settings")
        .find_one(
            doc! {
                "_id": user.id
            },
            FindOneOptions::builder()
                .projection(projection)
                .build()
        )
        .await
        .map_err(|_| Error::DatabaseError { operation: "find_one", with: "user_settings" })? {
        Ok(json!(doc))
    } else {
        Ok(json!({ }))
    }
}
