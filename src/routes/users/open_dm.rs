use revolt_quark::{Error, Result};

use mongodb::bson::doc;
use rocket::serde::json::Value;
use ulid::Ulid;

#[get("/<target>/dm")]
pub async fn req(/*user: UserRef, target: Ref*/ target: String) -> Result<Value> {
    todo!()
}
