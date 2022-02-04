use revolt_quark::{EmptyResponse, Result};

use chrono::prelude::*;
use mongodb::bson::{doc, to_bson};
use mongodb::options::UpdateOptions;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

type Data = HashMap<String, String>;

#[derive(FromForm, Serialize, Deserialize)]
pub struct Options {
    timestamp: Option<i64>,
}

#[post("/settings/set?<options..>", data = "<data>")]
pub async fn req(
    /*user: UserRef,*/ data: Json<Data>,
    options: Options,
) -> Result<EmptyResponse> {
    todo!()
}
