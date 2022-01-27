use revolt_quark::{Error, Result};

use rocket::serde::json::Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CreateUserResponse {
    token: String,
}

#[post("/<target>/join_call")]
pub async fn req(/*user: UserRef, target: Ref,*/ target: String) -> Result<Value> {
    todo!()
}
