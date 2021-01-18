use crate::database::*;

use mongodb::bson::{doc, from_document};
use rauth::auth::Session;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = rauth::util::Error;

    async fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let session: Session = try_outcome!(request.guard::<Session>().await);

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
                rauth::util::Error::DatabaseError,
            ))
        }
    }
}
