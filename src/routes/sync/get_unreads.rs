use crate::database::*;
use crate::util::result::Result;

use mongodb::bson::doc;
use rocket_contrib::json::JsonValue;

#[get("/unreads")]
pub async fn req(user: User) -> Result<JsonValue> {
    Ok(json!(User::fetch_unreads(&user.id).await?))
}
