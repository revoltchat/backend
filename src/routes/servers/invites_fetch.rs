use revolt_quark::{Error, Result};

use futures::StreamExt;
use mongodb::bson::{doc, from_document};
use rocket::serde::json::Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerInvite {
    #[serde(rename = "_id")]
    code: String,
    creator: String,
    channel: String,
}

#[get("/<target>/invites")]
pub async fn req(/*user: UserRef, target: Ref*/ target: String) -> Result<Value> {
    todo!()
}
