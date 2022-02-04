use mongodb::bson::doc;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use revolt_quark::{EmptyResponse, Result};

#[derive(Serialize, Deserialize)]
pub struct Values {
    server: u32,
    channel: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    permissions: Values,
}

#[put("/<target>/permissions/<role_id>", data = "<data>", rank = 2)]
pub async fn req(
    /*user: UserRef, target: Ref,*/ target: String,
    role_id: String,
    data: Json<Data>,
) -> Result<EmptyResponse> {
    todo!()
}
