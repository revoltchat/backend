use revolt_quark::{Error, Result};

use mongodb::bson::doc;
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    name: String,
}

#[post("/<target>/roles", data = "<data>")]
pub async fn req(
    /*user: UserRef, target: Ref,*/ target: String,
    data: Json<Data>,
) -> Result<Value> {
    todo!()
}
