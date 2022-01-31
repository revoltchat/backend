use revolt_quark::{Error, Result};

use rocket::serde::json::Value;

#[get("/<target>")]
pub async fn req(user: User, target: User, target: String) -> Result<Value> {
    todo!()
}
