use crate::util::regex::RE_USERNAME;
use mongodb::bson::doc;
use rauth::entities::Account;
use revolt_quark::{models::User, Database, EmptyResponse, Error, Result};
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    username: String,
    #[validate(length(min = 8, max = 1024))]
    password: String,
}

#[patch("/@me/username", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    account: Account,
    mut user: User,
    data: Json<Data>,
) -> Result<EmptyResponse> {
    let data = data.into_inner();
    account
        .verify_password(&data.password)
        .map_err(|_| Error::InvalidCredentials)?;
    user.update_username(db, data.username)
        .await
        .map(|_| EmptyResponse)
}
