use revolt_quark::{Error, Result};

use rocket::serde::json::Value;

#[get("/relationships")]
pub async fn req(/*user: UserRef*/) -> Result<Value> {
    todo!()
}
