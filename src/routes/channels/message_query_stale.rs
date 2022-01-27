use revolt_quark::{Error, Result};

use futures::StreamExt;
use mongodb::bson::{doc, from_document};
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Options {
    ids: Vec<String>,
}

#[post("/<target>/messages/stale", data = "<data>")]
pub async fn req(/*user: UserRef, target: Ref,*/ target: String, data: Json<Options>) -> Result<Value> {
    todo!()
}
