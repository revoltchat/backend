use crate::database::*;
use crate::util::result::{EmptyResponse, Error, Result};

use mongodb::bson::doc;
use rauth::entities::{Model, Session, WebPushSubscription};
use rocket::serde::json::Json;

#[post("/subscribe", data = "<data>")]
pub async fn req(mut session: Session, data: Json<WebPushSubscription>) -> Result<EmptyResponse> {
    session.subscription = Some(data.into_inner());
    session
        .save(&get_db(), None)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| Error::DatabaseError {
            operation: "save",
            with: "session",
        })
}
