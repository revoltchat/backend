use crate::*;

use rauth::entities::Session;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};

#[async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = rauth::util::Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let user: &Option<User> = request.local_cache_async(async {
            let header_bot_token = request
                .headers()
                .get("x-bot-token")
                .next()
                .map(|x| x.to_string());

            if let Some(bot_token) = header_bot_token {
                if let Ok(bot) = get_db().get_bot_from_token(bot_token.as_str()).await {
                    if let Ok(user) = get_db().get_user(bot.id.as_str()).await {
                        return Some(user)
                    }
                }
            } else {
                if let Outcome::Success(session) = request.guard::<Session>().await {
                    if let Ok(user) = get_db().get_user(session.user_id.as_str()).await {
                        return Some(user)
                    }
                }
            }

            None
        }).await;

        if let Some(user) = user {
            Outcome::Success(user.clone())
        } else {
            Outcome::Failure((
                Status::Forbidden,
                rauth::util::Error::InvalidSession,
            ))
        }
    }
}
