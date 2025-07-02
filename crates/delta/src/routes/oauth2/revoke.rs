use revolt_config::config;
use revolt_models::v0;
use rocket::{serde::json::Json, State};
use revolt_database::{util::oauth2::{self, TokenType}, AuthorizedBotId, Database, User};
use revolt_result::{create_error, Result};

/// Revoke an OAuth2 token
///
/// This can either take user authorization and a client_id,
/// or an OAuth2 token.
#[openapi(tag = "OAuth2")]
#[post("/revoke?<token>&<client_id>")]
pub async fn revoke(
    db: &State<Database>,
    user: Option<User>,
    token: Option<&str>,
    client_id: Option<&str>
) -> Result<Json<v0::AuthorizedBot>> {
    let config = config().await;

    let (user_id, bot_id) = match (user, client_id, token) {
        (Some(user), Some(client_id), None) => {
            (user.id.clone(), client_id.to_string())
        },
        (_, None, Some(token)) => {
            let Ok(claims) = oauth2::decode_token(&config.api.security.token_secret, token) else {
                return Err(create_error!(NotAuthenticated))
            };

            if claims.r#type == TokenType::Auth {
                return Err(create_error!(InvalidOperation))
            }

            (claims.sub, claims.client_id)
        },
        _ => return Err(create_error!(InvalidOperation))
    };

    let id = AuthorizedBotId { user: user_id, bot: bot_id };

    let authorized_bot = db.fetch_authorized_bot(&id).await?;

    if authorized_bot.deauthorized_at.is_some() {
        return Err(create_error!(InvalidOperation))
    }

    db.deauthorize_authorized_bot(&id)
        .await
        .map(|bot| Json(bot.into()))
}