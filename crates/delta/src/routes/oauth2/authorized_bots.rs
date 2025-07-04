use revolt_models::v0;
use rocket::{serde::json::Json, State};
use revolt_database::{Database, User};
use revolt_result::Result;

#[openapi(tag = "OAuth2")]
#[get("/authorized_bots")]
pub async fn authorized_bots(
    db: &State<Database>,
    user: User
) -> Result<Json<Vec<v0::AuthorizedBotsResponse>>> {
    let authorized_bots = db.fetch_users_authorized_bots(&user.id).await?;

    let mut response = Vec::new();

    for authorized_bot in authorized_bots {
        let bot = db.fetch_bot(&authorized_bot.id.bot).await?;
        let bot_user = db.fetch_user(&authorized_bot.id.bot).await?;

        response.push(v0::AuthorizedBotsResponse {
            authorized_bot: authorized_bot.into(),
            public_bot: bot.into_public_bot(bot_user)
        });
    };

    Ok(Json(response))
}