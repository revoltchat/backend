use revolt_quark::{Result, models::User, Ref};

use rocket::serde::json::Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CreateUserResponse {
    token: String,
}

#[post("/<_target>/join_call")]
pub async fn req(_user: User, _target: Ref) -> Result<Value> {
    todo!()
}
