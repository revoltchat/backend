use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};

use mongodb::bson::{doc, to_document};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<UserStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    profile: Option<UserProfile>,
}

#[patch("/", data = "<data>")]
pub async fn req(user: User, data: Json<Data>) -> Result<()> {
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    get_collection("users")
    .update_one(
        doc! { "_id": &user.id },
        doc! { "$set": to_document(&data.0).map_err(|_| Error::DatabaseError { operation: "to_document", with: "data" })? },
        None
    )
    .await
    .map_err(|_| Error::DatabaseError { operation: "update_one", with: "user" })?;

    ClientboundNotification::UserUpdate {
        id: user.id.clone(),
        data: json!(data.0),
    }
    .publish(user.id.clone())
    .await
    .ok();

    Ok(())
}
