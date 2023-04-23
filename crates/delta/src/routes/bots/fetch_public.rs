use revolt_database::Database;
use revolt_models::PublicBot;
use revolt_quark::{models::User, Error, Ref, Result};

use rocket::serde::json::Json;
use rocket::State;

/// # Fetch Public Bot
///
/// Fetch details of a public (or owned) bot by its id.
#[openapi(tag = "Bots")]
#[get("/<target>/invite")]
pub async fn fetch_public_bot(
    db: &State<Database>,
    user: Option<User>,
    target: Ref,
) -> Result<Json<PublicBot>> {
    let bot = db.fetch_bot(&target.id).await.map_err(Error::from_core)?;
    if !bot.public && user.map_or(true, |x| x.id != bot.owner) {
        return Err(Error::NotFound);
    }

    let user = db.fetch_user(&bot.id).await.map_err(Error::from_core)?;
    Ok(Json(PublicBot::from(bot, user)))
}
