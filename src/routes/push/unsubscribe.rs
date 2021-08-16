use crate::database::*;
use crate::util::result::{Error, Result, EmptyResponse};

use mongodb::bson::doc;
use rauth::auth::Session;

#[post("/unsubscribe")]
pub async fn req(session: Session) -> Result<EmptyResponse> {
    get_collection("accounts")
        .update_one(
            doc! {
                "_id": session.user_id,
                "sessions.id": session.id.unwrap()
            },
            doc! {
                "$unset": {
                    "sessions.$.subscription": 1
                }
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "to_document",
            with: "subscription",
        })?;

    Ok(EmptyResponse {})
}
