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
    #[validate]
    status: Option<UserStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate]
    profile: Option<UserProfile>,
}

#[patch("/<_ignore_id>", data = "<data>")]
pub async fn req(user: User, data: Json<Data>, _ignore_id: String) -> Result<()> {
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

    if let Some(status) = data.0.status {
        ClientboundNotification::UserUpdate {
            id: user.id.clone(),
            data: json!({ "status": status }),
        }
        .publish(user.id.clone())
        .await
        .ok();
    }

    Ok(())
}
