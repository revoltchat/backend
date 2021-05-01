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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, max = 128))]
    avatar: Option<String>,
}

#[patch("/<_ignore_id>", data = "<data>")]
pub async fn req(user: User, mut data: Json<Data>, _ignore_id: String) -> Result<()> {
    if data.0.status.is_none() && data.0.profile.is_none() && data.0.avatar.is_none() {
        return Ok(())
    }

    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let mut set = to_document(&data.0).map_err(|_| Error::DatabaseError { operation: "to_document", with: "data" })?;

    let avatar = std::mem::replace(&mut data.0.avatar, None);
    let attachment = if let Some(attachment_id) = avatar {
        let attachment = File::find_and_use(&attachment_id, "avatars", "user", &user.id).await?;
        set.insert("avatar", to_document(&attachment).map_err(|_| Error::DatabaseError { operation: "to_document", with: "attachment" })?);
        Some(attachment)
    } else {
        None
    };

    get_collection("users")
    .update_one(
        doc! { "_id": &user.id },
        doc! { "$set": set },
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

    if let Some(avatar) = attachment {
        ClientboundNotification::UserUpdate {
            id: user.id.clone(),
            data: json!({ "avatar": avatar }),
        }
        .publish(user.id.clone())
        .await
        .ok();

        if let Some(old_avatar) = user.avatar {
            old_avatar.delete().await?;
        }
    }

    Ok(())
}
