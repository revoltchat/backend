use revolt_quark::{Error, Result};

use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Options {
    keys: Vec<String>,
}

#[post("/settings/fetch", data = "<options>")]
pub async fn req(/*user: UserRef,*/ options: Json<Options>) -> Result<Value> {
    todo!()
}
