use revolt_quark::{EmptyResponse, Result};
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
pub async fn req(/*session: Session, user: Option<User>,*/ data: Json<Data>) -> Result<EmptyResponse> {
    todo!()
}
