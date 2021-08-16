use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result, EmptyResponse};
use crate::{database::*, notifications::events::RemoveUserField};

use mongodb::bson::{doc, to_document};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, Debug)]
pub struct UserProfileData {
    #[validate(length(min = 0, max = 2000))]
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, max = 128))]
    background: Option<String>,
}

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate]
    status: Option<UserStatus>,
    #[validate]
    profile: Option<UserProfileData>,
    #[validate(length(min = 1, max = 128))]
    avatar: Option<String>,
    remove: Option<RemoveUserField>,
}

#[patch("/<_ignore_id>", data = "<data>")]
pub async fn req(user: User, data: Json<Data>, _ignore_id: String) -> Result<EmptyResponse> {
    let mut data = data.into_inner();

    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if data.status.is_none()
        && data.profile.is_none()
        && data.avatar.is_none()
        && data.remove.is_none()
    {
        return Ok(EmptyResponse {});
    }

    let mut unset = doc! {};
    let mut set = doc! {};

    let mut remove_background = false;
    let mut remove_avatar = false;

    if let Some(remove) = &data.remove {
        match remove {
            RemoveUserField::ProfileContent => {
                unset.insert("profile.content", 1);
            }
            RemoveUserField::ProfileBackground => {
                unset.insert("profile.background", 1);
                remove_background = true;
            }
            RemoveUserField::StatusText => {
                unset.insert("status.text", 1);
            }
            RemoveUserField::Avatar => {
                unset.insert("avatar", 1);
                remove_avatar = true;
            }
        }
    }

    if let Some(status) = &data.status {
        set.insert(
            "status",
            to_document(&status).map_err(|_| Error::DatabaseError {
                operation: "to_document",
                with: "status",
            })?,
        );
    }

    if let Some(profile) = data.profile {
        if let Some(content) = profile.content {
            set.insert("profile.content", content);
        }

        if let Some(attachment_id) = profile.background {
            let attachment =
                File::find_and_use(&attachment_id, "backgrounds", "user", &user.id).await?;
            set.insert(
                "profile.background",
                to_document(&attachment).map_err(|_| Error::DatabaseError {
                    operation: "to_document",
                    with: "attachment",
                })?,
            );

            remove_background = true;
        }
    }

    let avatar = std::mem::replace(&mut data.avatar, None);
    if let Some(attachment_id) = avatar {
        let attachment = File::find_and_use(&attachment_id, "avatars", "user", &user.id).await?;
        set.insert(
            "avatar",
            to_document(&attachment).map_err(|_| Error::DatabaseError {
                operation: "to_document",
                with: "attachment",
            })?,
        );

        remove_avatar = true;
    }

    let mut operations = doc! {};
    if set.len() > 0 {
        operations.insert("$set", &set);
    }

    if unset.len() > 0 {
        operations.insert("$unset", unset);
    }

    if operations.len() > 0 {
        get_collection("users")
            .update_one(doc! { "_id": &user.id }, operations, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "user",
            })?;
    }

    ClientboundNotification::UserUpdate {
        id: user.id.clone(),
        data: json!(set),
        clear: data.remove,
    }
    .publish_as_user(user.id.clone());

    if remove_avatar {
        if let Some(old_avatar) = user.avatar {
            old_avatar.delete().await?;
        }
    }

    if remove_background {
        if let Some(profile) = user.profile {
            if let Some(old_background) = profile.background {
                old_background.delete().await?;
            }
        }
    }

    Ok(EmptyResponse {})
}
