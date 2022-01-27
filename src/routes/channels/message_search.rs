use revolt_quark::Result;

use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document},
    options::FindOptions,
};
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, FromFormField)]
pub enum Sort {
    Relevance,
    Latest,
    Oldest,
}

impl Default for Sort {
    fn default() -> Sort {
        Sort::Relevance
    }
}

#[derive(Validate, Serialize, Deserialize, FromForm)]
pub struct Options {
    #[validate(length(min = 1, max = 64))]
    query: String,

    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,
    #[validate(length(min = 26, max = 26))]
    before: Option<String>,
    #[validate(length(min = 26, max = 26))]
    after: Option<String>,
    #[serde(default = "Sort::default")]
    sort: Sort,
    include_users: Option<bool>,
}

#[post("/<target>/search", data = "<options>")]
pub async fn req(/*user: UserRef, target: Ref,*/ target: String, options: Json<Options>) -> Result<Value> {
    todo!()
}
