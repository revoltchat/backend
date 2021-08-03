use crate::database::*;
use crate::util::result::Result;

use mongodb::bson::doc;
use rocket::serde::json::Value;

#[get("/unreads")]
pub async fn req(user: User) -> Result<Value> {
    Ok(json!(User::fetch_unreads(&user.id).await?))
}
