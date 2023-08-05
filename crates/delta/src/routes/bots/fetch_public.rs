use revolt_database::{util::reference::Reference, Database, User};
use revolt_models::v0::PublicBot;
use revolt_result::{create_error, Result};

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
    target: Reference,
) -> Result<Json<PublicBot>> {
    let bot = db.fetch_bot(&target.id).await?;
    if !bot.public && user.map_or(true, |x| x.id != bot.owner) {
        return Err(create_error!(NotFound));
    }

    let user = db.fetch_user(&bot.id).await?;
    Ok(Json(bot.into_public_bot(user)))
}
