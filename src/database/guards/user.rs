use crate::database::*;

use mongodb::bson::{doc, from_document};
use rauth::entities::Session;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};

#[rocket::async_trait]
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
                if let Ok(id) = db_conn().get_user_id_by_bot_token(&bot_token).await {
                    if let Ok(user) = db_conn().get_user_by_id(&id).await {
                        return Some(user)
                    }
                }
            } else {
                if let Outcome::Success(session) = request.guard::<Session>().await {
                    if let Ok(user) = db_conn().get_user_by_id(&session.user_id).await {
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
