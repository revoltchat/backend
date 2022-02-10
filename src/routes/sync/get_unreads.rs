use revolt_quark::{Result, models::{ChannelUnread, User}, Db};

use mongodb::bson::doc;
use rocket::serde::json::Json;

#[get("/unreads")]
pub async fn req(db: &Db, user: User) -> Result<Json<Vec<ChannelUnread>>> {
    db.fetch_unreads(&user.id).await.map(Json)
}
