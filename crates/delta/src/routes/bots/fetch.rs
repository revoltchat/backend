use revolt_quark::{
    models::{Bot, User},
    Db, Error, Ref, Result,
};
use rocket::serde::json::Json;
use serde::Serialize;

/// # Bot Response
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
#[get("/<target>")]
pub async fn fetch_bot(db: &Db, user: User, target: Ref) -> Result<Json<BotResponse>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let bot = target.as_bot(db).await?;
    if bot.owner != user.id {
        return Err(Error::NotFound);
    }

    Ok(Json(BotResponse {
        user: db.fetch_user(&bot.id).await?.foreign(),
        bot,
    }))
}
