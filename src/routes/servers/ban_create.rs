use revolt_quark::{EmptyResponse, Result};

use mongodb::bson::doc;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 1024))]
    reason: Option<String>,
}

#[put("/<server>/bans/<target>", data = "<data>")]
pub async fn req(/*user: UserRef, server: Ref, target: Ref,*/ server: String, target: String, data: Json<Data>) -> Result<EmptyResponse> {
    todo!()
}
