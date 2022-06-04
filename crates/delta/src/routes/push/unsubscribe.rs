use revolt_quark::{
    rauth::{models::Session, RAuth},
    EmptyResponse, Error, Result,
};

use rocket::State;

/// # Unsubscribe
///
/// Remove the Web Push subscription associated with the current session.
#[openapi(tag = "Web Push")]
#[post("/unsubscribe")]
pub async fn req(rauth: &State<RAuth>, mut session: Session) -> Result<EmptyResponse> {
    session.subscription = None;
    session
        .save(&rauth)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| Error::DatabaseError {
            operation: "save",
            with: "session",
        })
}
