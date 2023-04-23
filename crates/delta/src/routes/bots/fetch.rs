use revolt_database::{util::reference::Reference, Database};
use revolt_models::FetchBotResponse;
use revolt_quark::{models::User, Error, Result};
use rocket::{serde::json::Json, State};

/// # Fetch Bot
///
/// Fetch details of a bot you own by its id.
#[openapi(tag = "Bots")]
#[get("/<bot>")]
pub async fn fetch_bot(
    db: &State<Database>,
    user: User,
    bot: Reference,
) -> Result<Json<FetchBotResponse>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let bot = bot.as_bot(db).await.map_err(Error::from_core)?;
    if bot.owner != user.id {
        return Err(Error::NotFound);
    }

    Ok(Json(FetchBotResponse {
        user: revolt_models::User::from(
            db.fetch_user(&bot.id).await.map_err(Error::from_core)?,
            None,
        )
        .await,
        bot: bot.into(),
    }))
}
