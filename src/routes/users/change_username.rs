use revolt_quark::{EmptyResponse, Result};
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
    //user: UserRef,
    data: Json<Data>,
    _ignore_id: String,
) -> Result<EmptyResponse> {
    todo!()
}
