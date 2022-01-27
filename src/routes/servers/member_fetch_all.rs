use revolt_quark::{Error, Result};

use mongodb::bson::{doc, from_document, Document};
use rocket::serde::json::Value;

#[get("/<target>/members")]
pub async fn req(/*user: UserRef, target: Ref*/ target: String) -> Result<Value> {
    todo!()
}
