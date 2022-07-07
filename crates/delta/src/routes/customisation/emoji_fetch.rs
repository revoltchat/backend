use revolt_quark::models::{Emoji, User};
use revolt_quark::{Db, Ref, Result};

use rocket::serde::json::Json;

/// # Fetch Emoji
///
/// Fetch an emoji by its id.
#[openapi(tag = "Emojis")]
#[get("/emoji/<id>")]
pub async fn fetch_emoji(db: &Db, _user: User, id: Ref) -> Result<Json<Emoji>> {
    id.as_emoji(db).await.map(Json)
}
