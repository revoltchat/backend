use crate::database::*;
use crate::util::result::Result;

use rocket::serde::json::Value;

#[get("/relationships")]
pub async fn req(user: User) -> Result<Value> {
    Ok(if let Some(vec) = user.relations {
        json!(vec)
    } else {
        json!([])
    })
}
