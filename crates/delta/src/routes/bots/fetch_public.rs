use revolt_database::Database;
use revolt_models::PublicBot;
use revolt_quark::{models::User, Db, Error, Ref, Result};

use rocket::serde::json::Json;
use rocket::State;

/// # Fetch Public Bot
///
/// Fetch details of a public (or owned) bot by its id.
#[openapi(tag = "Bots")]
#[get("/<target>/invite")]
pub async fn fetch_public_bot(
    legacy_db: &Db,
    db: &State<Database>,
    user: Option<User>,
    target: Ref,
) -> Result<Json<PublicBot>> {
    let bot = db.fetch_bot(&target.id).await.map_err(Error::from_core)?;
    if !bot.public && user.map_or(true, |x| x.id != bot.owner) {
        return Err(Error::NotFound);
    }

    let user = legacy_db.fetch_user(&bot.id).await?;

    Ok(Json(PublicBot::from(
        bot,
        user.username,
        user.avatar.map(|f| f.id),
        user.profile.and_then(|p| p.content),
    )))
}
