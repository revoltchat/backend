use crate::database::{db_conn, Queries};
use crate::util::result::{EmptyResponse, Error, Result};

use mongodb::bson::doc;
use rauth::entities::{Model, Session};

#[post("/unsubscribe")]
pub async fn req(mut session: Session) -> Result<EmptyResponse> {
    session.subscription = None;
    session
        .save(db_conn().get_db().await, None)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| Error::DatabaseError {
            operation: "save",
            with: "session",
        })
}
