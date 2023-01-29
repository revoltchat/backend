use revolt_quark::{
    authifier::{models::Session, Authifier},
    EmptyResponse, Error, Result,
};

use rocket::State;

/// # Unsubscribe
///
/// Remove the Web Push subscription associated with the current session.
#[openapi(tag = "Web Push")]
#[post("/unsubscribe")]
pub async fn req(authifier: &State<Authifier>, mut session: Session) -> Result<EmptyResponse> {
    session.subscription = None;
    session
        .save(authifier)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| Error::DatabaseError {
            operation: "save",
            with: "session",
        })
}
