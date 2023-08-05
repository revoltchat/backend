use revolt_database::{util::reference::Reference, Database, User};
use revolt_models::v0::FetchBotResponse;
use revolt_result::{create_error, Result};
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
        return Err(create_error!(IsBot));
    }

    let bot = bot.as_bot(db).await?;
    if bot.owner != user.id {
        return Err(create_error!(NotFound));
    }

    Ok(Json(FetchBotResponse {
        user: db.fetch_user(&bot.id).await?.into(None).await,
        bot: bot.into(),
    }))
}
