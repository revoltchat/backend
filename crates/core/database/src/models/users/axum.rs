use axum::{extract::FromRequestParts, http::request::Parts};

use revolt_config::config;
use revolt_models::v0;
use revolt_result::{create_error, Error, Result};

use crate::{util::oauth2, Database, OAuth2Scope, User};

#[async_trait::async_trait]
impl FromRequestParts<Database> for User {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, db: &Database) -> Result<User> {
        if let Some(Ok(bot_token)) = parts.headers.get("x-bot-token").map(|v| v.to_str()) {
            let bot = db.fetch_bot_by_token(bot_token).await?;
            db.fetch_user(&bot.id).await
        } else if let Some(Ok(session_token)) =
            parts.headers.get("x-session-token").map(|v| v.to_str())
        {
            let session = db.fetch_session_by_token(session_token).await?;
            db.fetch_user(&session.user_id).await
        } else if let Some(Ok(header_oauth_token)) = parts.headers.get("x-oauth2-token").map(|v| v.to_str()) {
            let config = config().await;

            let claims = oauth2::decode_token(
                &config.api.security.token_secret,
                header_oauth_token,
            ).map_err(|_| create_error!(NotAuthenticated))?;

            let required_scope: v0::OAuth2Scope = parts.extensions.get::<OAuth2Scope>()
                .copied()
                .ok_or_else(|| create_error!(NotAuthenticated))?
                .into();

            if claims.scopes.contains(&v0::OAuth2Scope::Full) || claims.scopes.contains(&required_scope) {
                db.fetch_user(&claims.sub).await
            } else {
                Err(create_error!(MissingScope { scope: required_scope.to_string() }))
            }
        } else {
            Err(create_error!(NotAuthenticated))
        }
    }
}
