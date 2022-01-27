use revolt_quark::{EmptyResponse, Result, models::channel::FieldsChannel};

use mongodb::bson::doc;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[validate(length(min = 0, max = 1024))]
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[validate(length(min = 1, max = 128))]
    icon: Option<String>,
    remove: Option<FieldsChannel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>
}

#[patch("/<target>", data = "<data>")]
pub async fn req(/*user: UserRef, target: Ref,*/ target: String, data: Json<Data>) -> Result<EmptyResponse> {
    todo!()
}
