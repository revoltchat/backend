use revolt_quark::{
    models::{ChannelUnread, User},
    Db, Result,
};

use rocket::serde::json::Json;

/// # Fetch Unreads
///
/// Fetch information about unread state on channels.
#[openapi(tag = "Sync")]
#[get("/unreads")]
pub async fn req(db: &Db, user: User) -> Result<Json<Vec<ChannelUnread>>> {
    db.fetch_unreads(&user.id).await.map(Json)
}
