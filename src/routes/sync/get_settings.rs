use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Options {
    keys: Vec<String>,
}

#[post("/settings/fetch", data = "<options>")]
pub async fn req(user: User, options: Json<Options>) -> Result<Value> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }
    
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
            FindOneOptions::builder().projection(projection).build(),
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find_one",
            with: "user_settings",
        })?
    {
        Ok(json!(doc))
    } else {
        Ok(json!({}))
    }
}
