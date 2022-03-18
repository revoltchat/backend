use revolt_quark::{
    models::{Bot, User},
    Db, Error, Result,
};
use rocket::serde::json::Json;

/// # Fetch Owned Bots
///
/// Fetch all of the bots that you have control over.
#[openapi(tag = "Bots")]
#[get("/@me")]
pub async fn fetch_owned_bots(db: &Db, user: User) -> Result<Json<Vec<Bot>>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    db.fetch_bots_by_user(&user.id).await.map(Json)
}
