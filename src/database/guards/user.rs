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
                if let Ok(result) = get_collection("bots")
                    .find_one(
                        doc! {
                            "token": bot_token
                        },
                        None,
                    )
                    .await
                {
                    if let Some(doc) = result {
                        let id = doc.get_str("_id").unwrap();
                        if let Ok(result) = get_collection("users")
                            .find_one(
                                doc! {
                                    "_id": &id
                                },
                                None,
                            )
                            .await
                        {
                            if let Some(doc) = result {
                                if let Ok(user) = from_document(doc) {
                                    return Some(user)
                                }
                            }
                        }
                    }
                }
            } else {
                if let Outcome::Success(session) = request.guard::<Session>().await {
                    if let Ok(result) = get_collection("users")
                        .find_one(
                            doc! {
                                "_id": &session.user_id
                            },
                            None,
                        )
                        .await
                    {
                        if let Some(doc) = result {
                            if let Ok(user) = from_document(doc) {
                                return Some(user)
                            }
                        }
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
