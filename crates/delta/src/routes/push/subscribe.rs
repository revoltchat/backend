use revolt_quark::{
    rauth::{
        models::{Session, WebPushSubscription},
        RAuth,
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
    rauth: &State<RAuth>,
    mut session: Session,
    data: Json<WebPushSubscription>,
) -> Result<EmptyResponse> {
    session.subscription = Some(data.into_inner());
    session
        .save(&rauth)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| Error::DatabaseError {
            operation: "save",
            with: "session",
        })
}
