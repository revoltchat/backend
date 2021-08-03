use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;
use rauth::auth::Session;
use regex::Regex;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

lazy_static! {
    static ref RE_USERNAME: Regex = Regex::new(r"^[a-zA-Z0-9_.]+$").unwrap();
}

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    username: String,
}

#[post("/complete", data = "<data>")]
pub async fn req(session: Session, user: Option<User>, data: Json<Data>) -> Result<()> {
    if user.is_some() {
        Err(Error::AlreadyOnboarded)?
    }

    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if User::is_username_taken(&data.username).await? {
        return Err(Error::UsernameTaken);
    }

    get_collection("users")
        .insert_one(
            doc! {
                "_id": session.user_id,
                "username": &data.username
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "insert_one",
            with: "user",
        })?;

    Ok(())
}
