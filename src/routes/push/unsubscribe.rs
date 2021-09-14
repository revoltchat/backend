use crate::database::*;
use crate::util::result::{EmptyResponse, Error, Result};

use mongodb::bson::doc;
use rauth::entities::{Model, Session};

#[post("/unsubscribe")]
pub async fn req(mut session: Session) -> Result<EmptyResponse> {
    session.subscription = None;
    session
        .save(&get_db(), None)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| Error::DatabaseError {
            operation: "save",
            with: "session",
        })
}
