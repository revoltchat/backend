use mongodb::bson::doc;
use rocket::serde::json::Json;
use serde::{Serialize, Deserialize};

use revolt_quark::{EmptyResponse, Result};

#[derive(Serialize, Deserialize)]
pub struct Values {
    server: u32,
    channel: u32
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    permissions: Values
}

#[put("/<target>/permissions/default", data = "<data>", rank = 1)]
pub async fn req(/*user: UserRef, target: Ref,*/ target: String, data: Json<Data>) -> Result<EmptyResponse> {
    todo!()
}
