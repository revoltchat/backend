use revolt_quark::{Error, Result};
use revolt_quark::models::File;

use futures::StreamExt;
use mongodb::options::FindOptions;
use serde::{Serialize, Deserialize};
use rocket::serde::json::Value;
use mongodb::bson::{doc, from_document};

#[derive(Serialize, Deserialize)]
struct BannedUser {
    _id: String,
    username: String,
    avatar: Option<File>
}

#[get("/<target>/bans")]
pub async fn req(/*user: UserRef, target: Ref*/ target: String) -> Result<Value> {
    todo!()
}
