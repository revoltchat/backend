use revolt_quark::models::{Emoji, User};
use revolt_quark::{perms, Db, Ref, Result};

use rocket::serde::json::Json;

/// # Fetch Server Emoji
///
/// Fetch all emoji on a server.
#[openapi(tag = "Server Customisation")]
#[get("/<target>/emojis")]
pub async fn list_emoji(db: &Db, user: User, target: Ref) -> Result<Json<Vec<Emoji>>> {
    let server = target.as_server(db).await?;
    perms(&user).server(&server).calc(db).await?;

    // Fetch all emoji from server if we can view it
    db.fetch_emoji_by_parent_id(&server.id).await.map(Json)
}
