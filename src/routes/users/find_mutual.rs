use revolt_quark::{Error, Result};

use futures::StreamExt;
use mongodb::bson::{doc, Document};
use mongodb::options::FindOptions;
use rocket::serde::json::Value;

#[get("/<target>/mutual")]
pub async fn req(/*user: UserRef, target: Ref*/ target: String) -> Result<Value> {
    todo!()
}
