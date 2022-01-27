use crate::util::regex::RE_USERNAME;

use revolt_quark::Result;

use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    name: String,
}

#[post("/create", data = "<info>")]
pub async fn create_bot(/* user: User ,*/ info: Json<Data>) -> Result<Value> {
    todo!()
}
