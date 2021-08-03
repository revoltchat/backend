use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::{doc, to_document};
use rauth::auth::Session;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Subscription {
    endpoint: String,
    p256dh: String,
    auth: String,
}

#[post("/subscribe", data = "<data>")]
pub async fn req(session: Session, data: Json<Subscription>) -> Result<()> {
    let data = data.into_inner();
    get_collection("accounts")
        .update_one(
            doc! {
                "_id": session.user_id,
                "sessions.id": session.id.unwrap()
            },
            doc! {
                "$set": {
                    "sessions.$.subscription": to_document(&data)
                        .map_err(|_| Error::DatabaseError { operation: "to_document", with: "subscription" })?
                }
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError { operation: "update_one", with: "account" })?;

    Ok(())
}
