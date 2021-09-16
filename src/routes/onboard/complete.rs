use crate::database::*;
use crate::util::result::{Error, Result, EmptyResponse};
use crate::util::regex::RE_USERNAME;

use mongodb::bson::doc;
use rauth::entities::Session;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    username: String,
}

#[post("/complete", data = "<data>")]
pub async fn req(session: Session, user: Option<User>, data: Json<Data>) -> Result<EmptyResponse> {
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

    Ok(EmptyResponse {})
}
