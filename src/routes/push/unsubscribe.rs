use revolt_quark::{EmptyResponse, Error, Result};

use rauth::{
    entities::{Model, Session},
    logic::Auth,
};
use rocket::State;

/// # Unsubscribe
///
/// Remove the Web Push subscription associated with the current session.
#[openapi(tag = "Web Push")]
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
