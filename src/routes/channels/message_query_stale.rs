use revolt_quark::{models::User, Ref, Result};

use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Options {
    ids: Vec<String>,
}

#[post("/<_target>/messages/stale", data = "<_data>")]
pub async fn req(_user: User, _target: Ref, _data: Json<Options>) -> Result<Value> {
    unimplemented!()
}
