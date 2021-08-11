use crate::database::*;

use mongodb::bson::{doc, from_document};
use rauth::auth::Session;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = rauth::util::Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let header_bot_token = request
            .headers()
            .get("x-bot-token")
            .next()
            .map(|x| x.to_string());
        
        if let Some(bot_token) = header_bot_token {
            return if let Ok(result) = get_collection("bots")
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
                            Outcome::Success(from_document(doc).unwrap())
                        } else {
                            Outcome::Failure((Status::Forbidden, rauth::util::Error::InvalidSession))
                        }
                    } else {
                        Outcome::Failure((
                            Status::InternalServerError,
                            rauth::util::Error::DatabaseError {
                                operation: "find_one",
                                with: "user",
                            },
                        ))
                    }
                } else {
                    Outcome::Failure((Status::Forbidden, rauth::util::Error::InvalidSession))
                }
            } else {
                Outcome::Failure((
                    Status::InternalServerError,
                    rauth::util::Error::DatabaseError {
                        operation: "find_one",
                        with: "bot",
                    },
                ))
            }
        }

        let session: Session = request.guard::<Session>().await.unwrap();

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
                Outcome::Success(from_document(doc).unwrap())
            } else {
                Outcome::Failure((Status::Forbidden, rauth::util::Error::InvalidSession))
            }
        } else {
            Outcome::Failure((
                Status::InternalServerError,
                rauth::util::Error::DatabaseError {
                    operation: "find_one",
                    with: "user",
                },
            ))
        }
    }
}
