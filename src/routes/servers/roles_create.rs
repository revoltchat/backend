use revolt_quark::{Error, Result};

use ulid::Ulid;
use mongodb::bson::doc;
use validator::Validate;
use serde::{Serialize, Deserialize};
use rocket::serde::json::{Json, Value};

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    name: String
}

#[post("/<target>/roles", data = "<data>")]
pub async fn req(/*user: UserRef, target: Ref,*/ target: String, data: Json<Data>) -> Result<Value> {
    todo!()
}
