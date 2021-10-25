use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result, EmptyResponse};
use crate::util::regex::RE_USERNAME;
use mongodb::bson::doc;
use rauth::entities::Account;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    username: Option<String>,
    #[validate(length(min = 8, max = 1024))]
    password: String,
}

#[patch("/<_ignore_id>/username", data = "<data>")]
pub async fn req(
    account: Account,
    user: User,
    data: Json<Data>,
    _ignore_id: String,
) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }

    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    account.verify_password(&data.password)
        .map_err(|_| Error::InvalidCredentials)?;

    let mut set = doc! {};
    if let Some(username) = &data.username {
        if (username.to_lowercase() != user.username.to_lowercase()) && User::is_username_taken(&username).await? {
            return Err(Error::UsernameTaken);
        }

        set.insert("username", username.clone());
    }

    get_collection("users")
        .update_one(doc! { "_id": &user.id }, doc! { "$set": set }, None)
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "update_one",
            with: "user",
        })?;

    ClientboundNotification::UserUpdate {
        id: user.id.clone(),
        data: json!({
            "username": data.username
        }),
        clear: None,
    }
    .publish_as_user(user.id.clone());

    Ok(EmptyResponse {})
}
