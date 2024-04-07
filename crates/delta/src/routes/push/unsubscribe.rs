use authifier::{models::Session, Authifier};

use revolt_result::{create_database_error, Result};
use rocket_empty::EmptyResponse;

use rocket::State;

/// # Unsubscribe
///
/// Remove the Web Push subscription associated with the current session.
#[openapi(tag = "Web Push")]
#[post("/unsubscribe")]
pub async fn unsubscribe(
    authifier: &State<Authifier>,
    mut session: Session,
) -> Result<EmptyResponse> {
    session.subscription = None;
    session
        .save(authifier)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| create_database_error!("save", "session"))
}
