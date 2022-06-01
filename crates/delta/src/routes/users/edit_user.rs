use revolt_quark::models::user::{FieldsUser, PartialUser, User};
use revolt_quark::models::File;
use revolt_quark::{Database, Error, Result};

use revolt_quark::models::user::UserStatus;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// # Profile Data
#[derive(Validate, Serialize, Deserialize, Debug, JsonSchema)]
pub struct UserProfileData {
    /// Text to set as user profile description
    #[validate(length(min = 0, max = 2000))]
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    /// Attachment Id for background
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, max = 128))]
    background: Option<String>,
}

/// # User Data
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataEditUser {
    /// New user status
    #[validate]
    status: Option<UserStatus>,
    /// New user profile data
    ///
    /// This is applied as a partial.
    #[validate]
    profile: Option<UserProfileData>,
    /// Attachment Id for avatar
    #[validate(length(min = 1, max = 128))]
    avatar: Option<String>,
    /// Fields to remove from user object
    #[validate(length(min = 1))]
    remove: Option<Vec<FieldsUser>>,
}

/// # Edit User
///
/// Edit currently authenticated user.
#[openapi(tag = "User Information")]
#[patch("/@me", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    mut user: User,
    data: Json<DataEditUser>,
) -> Result<Json<User>> {
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

    // 1. Remove fields from object
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

    // 2. Apply new avatar
    if let Some(avatar) = data.avatar {
        partial.avatar = Some(File::use_avatar(db, &avatar, &user.id).await?);
    }

    // 3. Apply new status
    if let Some(status) = data.status {
        let mut new_status = user.status.take().unwrap_or_default();
        if let Some(text) = status.text {
            new_status.text = Some(text);
        }

        if let Some(presence) = status.presence {
            new_status.presence = Some(presence);
        }

        partial.status = Some(new_status);
    }

    // 4. Apply new profile
    if let Some(profile) = data.profile {
        let mut new_profile = user.profile.take().unwrap_or_default();
        if let Some(content) = profile.content {
            new_profile.content = Some(content);
        }

        if let Some(background) = profile.background {
            new_profile.background = Some(File::use_background(db, &background, &user.id).await?);
        }

        partial.profile = Some(new_profile);
    }

    user.update(db, partial, data.remove.unwrap_or_default())
        .await?;

    Ok(Json(user.foreign()))
}
