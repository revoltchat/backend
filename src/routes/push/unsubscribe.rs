use revolt_quark::{EmptyResponse, Error, Result};

use mongodb::bson::doc;
use rauth::{
    entities::{Model, Session},
    logic::Auth,
};
use rocket::State;

#[post("/unsubscribe")]
pub async fn req(auth: &State<Auth>, mut session: Session) -> Result<EmptyResponse> {
    session.subscription = None;
    session
        .save(&auth.db, None)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| Error::DatabaseError {
            operation: "save",
            with: "session",
        })
}
