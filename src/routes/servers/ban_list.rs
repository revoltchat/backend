use revolt_quark::models::File;
use revolt_quark::{Error, Result};

use futures::StreamExt;
use mongodb::bson::{doc, from_document};
use mongodb::options::FindOptions;
use rocket::serde::json::Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct BannedUser {
    _id: String,
    username: String,
    avatar: Option<File>,
}

#[get("/<target>/bans")]
pub async fn req(/*user: UserRef, target: Ref*/ target: String) -> Result<Value> {
    todo!()
}
