use revolt_database::{util::reference::Reference, Database};
use revolt_models::Bot;
use revolt_quark::{models::User, Db, Error, Result};
use rocket::{serde::json::Json, State};
use serde::Serialize;

/// # Bot Response
/// TODO: move to revolt-models
#[derive(Serialize, JsonSchema)]
pub struct BotResponse {
    /// Bot object
    bot: Bot,
    /// User object
    user: User,
}

/// # Fetch Bot
///
/// Fetch details of a bot you own by its id.
#[openapi(tag = "Bots")]
#[get("/<bot>")]
pub async fn fetch_bot(
    legacy_db: &Db,
    db: &State<Database>,
    user: User,
    bot: Reference,
) -> Result<Json<BotResponse>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let bot = bot.as_bot(db).await.map_err(Error::from_core)?;
    if bot.owner != user.id {
        return Err(Error::NotFound);
    }

    Ok(Json(BotResponse {
        user: legacy_db.fetch_user(&bot.id).await?.foreign(),
        bot: bot.into(),
    }))
}
