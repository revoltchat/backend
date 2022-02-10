use revolt_quark::{EmptyResponse, Error, Result};

use mongodb::bson::doc;
use rauth::{
    entities::{Model, Session, WebPushSubscription},
    logic::Auth,
};
use rocket::{serde::json::Json, State};

#[post("/subscribe", data = "<data>")]
pub async fn req(
    auth: &State<Auth>,
    mut session: Session,
    data: Json<WebPushSubscription>,
) -> Result<EmptyResponse> {
    session.subscription = Some(data.into_inner());
    session
        .save(&auth.db, None)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| Error::DatabaseError {
            operation: "save",
            with: "session",
        })
}
