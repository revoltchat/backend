use crate::database::*;
use crate::util::result::{Result};

use rocket::serde::json::Value;

#[get("/@me")]
pub async fn req(user: User) -> Result<Value> {
    Ok(json!(user))
}
