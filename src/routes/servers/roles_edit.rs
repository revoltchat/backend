use revolt_quark::{EmptyResponse, Result};

use mongodb::bson::doc;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    name: Option<String>,
    #[validate(length(min = 1, max = 32))]
    colour: Option<String>,
    hoist: Option<bool>,
    rank: Option<i64>,
    // remove: Option<FieldsRole>,
}

#[patch("/<target>/roles/<role_id>", data = "<data>")]
pub async fn req(
    /*user: UserRef, target: Ref,*/ target: String,
    role_id: String,
    data: Json<Data>,
) -> Result<EmptyResponse> {
    todo!()
}
