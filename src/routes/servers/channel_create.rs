use std::collections::HashMap;

use revolt_quark::{Error, Result};

use mongodb::bson::doc;
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Serialize, Deserialize)]
enum ChannelType {
    Text,
    Voice,
}

impl Default for ChannelType {
    fn default() -> Self {
        ChannelType::Text
    }
}

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[serde(rename = "type", default = "ChannelType::default")]
    channel_type: ChannelType,
    #[validate(length(min = 1, max = 32))]
    name: String,
    #[validate(length(min = 0, max = 1024))]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
}

#[post("/<target>/channels", data = "<info>")]
pub async fn req(
    /*_idempotency: IdempotencyKey, user: User, target: Ref,*/ target: String,
    info: Json<Data>,
) -> Result<Value> {
    todo!()
}
