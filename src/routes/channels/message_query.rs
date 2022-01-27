use std::collections::HashSet;

use revolt_quark::{Error, Result};

use futures::{StreamExt, try_join};
use mongodb::{
    bson::{doc, from_document},
    options::FindOptions,
};
use rocket::serde::json::Value;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, FromFormField)]
pub enum Sort {
    Latest,
    Oldest,
}

#[derive(Validate, Serialize, Deserialize, FromForm)]
pub struct Options {
    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,
    #[validate(length(min = 26, max = 26))]
    before: Option<String>,
    #[validate(length(min = 26, max = 26))]
    after: Option<String>,
    sort: Option<Sort>,
    // Specifying 'nearby' ignores 'before', 'after' and 'sort'.
    // It will also take half of limit rounded as the limits to each side.
    // It also fetches the message ID specified.
    #[validate(length(min = 26, max = 26))]
    nearby: Option<String>,
    include_users: Option<bool>,
}

#[get("/<target>/messages?<options..>")]
pub async fn req(/*user: UserRef, target: Ref,*/ target: String, options: Options) -> Result<Value> {
    todo!()
}
