use crate::database::*;
use crate::util::result::{Error, Result};
use crate::util::variables::MAX_BOT_COUNT;

use mongodb::bson::{doc, to_document};
use regex::Regex;
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use nanoid::nanoid;
use validator::Validate;

// ! FIXME: should be global somewhere; maybe use config(?)
// ! tip: CTRL + F, RE_USERNAME
lazy_static! {
    static ref RE_USERNAME: Regex = Regex::new(r"^[a-zA-Z0-9_.]+$").unwrap();
}

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    name: String,
}

#[post("/create", data = "<info>")]
pub async fn create_bot(user: User, info: Json<Data>) -> Result<Value> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }
    
    let info = info.into_inner();
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if get_collection("bots")
        .count_documents(
            doc! {
                "owner": &user.id
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "count_documents",
            with: "bots",
        })? as usize >= *MAX_BOT_COUNT {
        return Err(Error::ReachedMaximumBots)
    }

    let id = Ulid::new().to_string();
    let token = nanoid!(64);
    let bot = Bot {
        id: id.clone(),
        owner: user.id.clone(),
        token,
        public: false,
        interactions_url: None
    };

    if User::is_username_taken(&info.name).await? {
        return Err(Error::UsernameTaken);
    }

    get_collection("users")
        .insert_one(
            doc! {
                "_id": &id,
                "username": &info.name,
                "bot": {
                    "owner": &user.id
                }
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "insert_one",
            with: "user",
        })?;

    get_collection("bots")
        .insert_one(
            to_document(&bot).map_err(|_| Error::DatabaseError { with: "bot", operation: "to_document" })?,
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "insert_one",
            with: "user",
        })?;

    Ok(json!(bot))
}
