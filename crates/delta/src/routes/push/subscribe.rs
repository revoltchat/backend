use authifier::{
    models::{Session, WebPushSubscription},
    Authifier,
};
use revolt_result::{create_database_error, Result};
use rocket::{serde::json::Json, State};
use rocket_empty::EmptyResponse;

/// # Push Subscribe
///
/// Create a new Web Push subscription.
///
/// If an existing subscription exists on this session, it will be removed.
#[openapi(tag = "Web Push")]
#[post("/subscribe", data = "<data>")]
pub async fn subscribe(
    authifier: &State<Authifier>,
    mut session: Session,
    data: Json<WebPushSubscription>,
) -> Result<EmptyResponse> {
    session.subscription = Some(data.into_inner());
    session
        .save(authifier)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| create_database_error!("save", "session"))
}
