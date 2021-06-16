use crate::database::*;
use crate::util::result::Result;

use rocket_contrib::json::JsonValue;
use mongodb::bson::doc;

#[get("/unreads")]
pub async fn req(user: User) -> Result<JsonValue> {
    Ok(json!(user.fetch_unreads().await?))
}
