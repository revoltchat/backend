use mongodb::bson::doc;
use rocket::serde::json::Json;
use serde::{Serialize, Deserialize};
use validator::Contains;

use revolt_quark::{EmptyResponse, Result};

#[derive(Serialize, Deserialize)]
pub struct Data {
    permissions: u32
}

#[put("/<target>/permissions/<role>", data = "<data>", rank = 2)]
pub async fn req(/*user: UserRef, target: Ref,*/ target: String, role: String, data: Json<Data>) -> Result<EmptyResponse> {
    todo!()
}
