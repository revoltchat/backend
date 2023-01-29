use revolt_quark::{
    authifier::{
        models::{Session, WebPushSubscription},
        Authifier,
    },
    EmptyResponse, Error, Result,
};

use rocket::{serde::json::Json, State};

/// # Push Subscribe
///
/// Create a new Web Push subscription.
///
/// If an existing subscription exists on this session, it will be removed.
#[openapi(tag = "Web Push")]
#[post("/subscribe", data = "<data>")]
pub async fn req(
    authifier: &State<Authifier>,
    mut session: Session,
    data: Json<WebPushSubscription>,
) -> Result<EmptyResponse> {
    session.subscription = Some(data.into_inner());
    session
        .save(authifier)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| Error::DatabaseError {
            operation: "save",
            with: "session",
        })
}
