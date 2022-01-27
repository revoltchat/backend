use revolt_quark::{Error, Result};

use mongodb::bson::doc;
use rocket::serde::json::Value;

#[get("/<target>/profile")]
pub async fn req(/*user: UserRef, target: Ref*/ target: String) -> Result<Value> {
    todo!()
}
