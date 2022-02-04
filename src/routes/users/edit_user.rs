//! Edit currently authenticated user.

use revolt_quark::models::user::{FieldsUser, PartialUser, User};
use revolt_quark::models::File;
use revolt_quark::{Database, Error, Result};

use mongodb::bson::doc;
use revolt_quark::models::user::UserStatus;
use rocket::serde::json::Json;
use rocket::State;
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
    #[validate(length(min = 1))]
    remove: Option<Vec<FieldsUser>>,
}

#[patch("/@me", data = "<data>")]
pub async fn req(db: &State<Database>, mut user: User, data: Json<Data>) -> Result<Json<User>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if data.status.is_none()
        && data.profile.is_none()
        && data.avatar.is_none()
        && data.remove.is_none()
    {
        return Ok(Json(user));
    }

    if let Some(fields) = &data.remove {
        if fields.contains(&FieldsUser::Avatar) {
            if let Some(avatar) = &user.avatar {
                db.mark_attachment_as_deleted(&avatar.id).await?;
            }
        }

        if fields.contains(&FieldsUser::ProfileBackground) {
            if let Some(profile) = &user.profile {
                if let Some(background) = &profile.background {
                    db.mark_attachment_as_deleted(&background.id).await?;
                }
            }
        }

        for field in fields {
            user.remove(field);
        }
    }

    let mut partial: PartialUser = Default::default();

    if let Some(avatar) = data.avatar {
        partial.avatar = Some(File::use_avatar(db, &avatar, &user.id).await?);
        user.avatar = partial.avatar.clone();
    }

    if let Some(status) = data.status {
        let mut new_status = user.status.take().unwrap_or_default();
        if let Some(text) = status.text {
            new_status.text = Some(text);
        }

        if let Some(presence) = status.presence {
            new_status.presence = Some(presence);
        }

        partial.status = Some(new_status);
        user.status = partial.status.clone();
    }

    if let Some(profile) = data.profile {
        let mut new_profile = user.profile.take().unwrap_or_default();
        if let Some(content) = profile.content {
            new_profile.content = Some(content);
        }

        if let Some(background) = profile.background {
            new_profile.background = Some(File::use_background(db, &background, &user.id).await?);
        }

        partial.profile = Some(new_profile);
        user.profile = partial.profile.clone();
    }

    db.update_user(&user.id, &partial, data.remove.unwrap_or_default())
        .await?;
    Ok(Json(user.foreign()))
}
