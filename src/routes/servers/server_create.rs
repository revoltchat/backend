use std::collections::HashMap;

use revolt_quark::{EmptyResponse, Result};

use mongodb::bson::doc;
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    name: String,
    #[validate(length(min = 0, max = 1024))]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>
}

#[post("/create", data = "<info>")]
pub async fn req(/*_idempotency: IdempotencyKey, user: User,*/ info: Json<Data>) -> Result<Value> {
    todo!()
}
