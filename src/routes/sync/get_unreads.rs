use revolt_quark::{Error, Result};

use mongodb::bson::doc;
use rocket::serde::json::Value;

#[get("/unreads")]
pub async fn req(/*user: UserRef*/) -> Result<Value> {
    todo!()
}
