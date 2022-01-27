use revolt_quark::{Error, Result};

use futures::try_join;
use mongodb::bson::doc;
use rocket::serde::json::Value;

#[delete("/<target>/friend")]
pub async fn req(/*user: UserRef, target: Ref*/ target: String) -> Result<Value> {
    todo!()
}
