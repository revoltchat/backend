use revolt_database::{Database, User};
use revolt_models::v0;
use revolt_result::Result;
use rocket::serde::json::Json;
use rocket::State;

/// # Fetch Unreads
///
/// Fetch information about unread state on channels.
#[openapi(tag = "Sync")]
#[get("/unreads")]
pub async fn unreads(db: &State<Database>, user: User) -> Result<Json<Vec<v0::ChannelUnread>>> {
    db.fetch_unreads(&user.id)
        .await
        .map(|v| v.into_iter().map(|u| u.into()).collect())
        .map(Json)
}
