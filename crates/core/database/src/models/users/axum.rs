use axum::{extract::FromRequestParts, http::request::Parts};

use revolt_result::{create_error, Error, Result};

use crate::{Database, User};

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
        } else {
            Err(create_error!(InvalidCredentials))
        }
    }
}
