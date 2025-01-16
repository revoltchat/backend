use revolt_quark::models::user::{FieldsUser, PartialUser, User};
use revolt_quark::models::File;
use revolt_quark::{Database, Error, Ref, Result};

use revolt_quark::models::user::UserStatus;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::util::regex::RE_DISPLAY_NAME;

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
    #[validate(length(min = 0, max = 2000))]
    #[serde(skip_serializing_if = "Option::is_none")]
    first_name: Option<String>,
    /// Last name
    #[validate(length(min = 0, max = 2000))]
    #[serde(skip_serializing_if = "Option::is_none")]
    last_name: Option<String>,
    /// Phone number
    #[validate(length(min = 0, max = 2000))]
    #[serde(skip_serializing_if = "Option::is_none")]
    phone_number: Option<String>,
    /// Country
    #[validate(length(min = 0, max = 2000))]
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<String>,
    /// City
    #[validate(length(min = 0, max = 2000))]
    #[serde(skip_serializing_if = "Option::is_none")]
    city: Option<String>,
    /// Occupation
    #[validate(length(min = 0, max = 2000))]
    #[serde(skip_serializing_if = "Option::is_none")]
    occupation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    x_account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    facebook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instagram: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tik_tok: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    relationship_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    likes_attending_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    favorite_destinations: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    languages_spoken: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    passions_and_hobbies: Option<String>,
}

/// # User Data
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataEditUser {
    /// New display name
    #[validate(length(min = 2, max = 32), regex = "RE_DISPLAY_NAME")]
    display_name: Option<String>,
    /// Attachment Id for avatar
    #[validate(length(min = 1, max = 128))]
    avatar: Option<String>,
    temporary_password: Option<bool>,

    /// New user status
    #[validate]
    status: Option<UserStatus>,
    /// New user profile data
    ///
    /// This is applied as a partial.
    #[validate]
    profile: Option<UserProfileData>,

    /// Bitfield of user badges
    #[serde(skip_serializing_if = "Option::is_none")]
    badges: Option<i32>,
    /// Enum of user flags
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<i32>,

    /// Fields to remove from user object
    #[validate(length(min = 1))]
    remove: Option<Vec<FieldsUser>>,
}

/// # Edit User
///
/// Edit currently authenticated user.
#[openapi(tag = "User Information")]
#[patch("/<target>", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    mut user: User,
    target: Ref,
    data: Json<DataEditUser>,
) -> Result<Json<User>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    // If we want to edit a different user than self, ensure we have
    // permissions and subsequently replace the user in question
    if target.id != "@me" && target.id != user.id {
        let target_user = target.as_user(db).await?;
        let is_bot_owner = target_user
            .bot
            .map(|bot| bot.owner == user.id)
            .unwrap_or_default();

        if !is_bot_owner && !user.privileged {
            return Err(Error::NotPrivileged);
        }
    }

    // Otherwise, filter out invalid edit fields
    if !user.privileged && (data.badges.is_some() || data.flags.is_some()) {
        return Err(Error::NotPrivileged);
    }

    // Exit out early if nothing is changed
    if data.display_name.is_none()
        && data.status.is_none()
        && data.profile.is_none()
        && data.avatar.is_none()
        && data.badges.is_none()
        && data.flags.is_none()
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

    let mut partial: PartialUser = PartialUser {
        display_name: data.display_name,
        badges: data.badges,
        flags: data.flags,
        ..Default::default()
    };

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

        if let Some(first_name) = profile.first_name {
            new_profile.first_name = Some(first_name);
        }

        if let Some(last_name) = profile.last_name {
            new_profile.last_name = Some(last_name);
        }

        if let Some(phone_number) = profile.phone_number {
            new_profile.phone_number = Some(phone_number);
        }

        if let Some(country) = profile.country {
            new_profile.country = Some(country);
        }

        if let Some(city) = profile.city {
            new_profile.city = Some(city);
        }

        if let Some(occupation) = profile.occupation {
            new_profile.occupation = Some(occupation);
        }
        if let Some(x_account) = profile.x_account {
            new_profile.x_account = Some(x_account);
        }
        if let Some(instagram) = profile.instagram {
            new_profile.instagram = Some(instagram);
        }
        if let Some(facebook) = profile.facebook {
            new_profile.facebook = Some(facebook);
        }
        if let Some(tik_tok) = profile.tik_tok {
            new_profile.tik_tok = Some(tik_tok);
        }
        if let Some(gender) = profile.gender {
            new_profile.gender = Some(gender);
        }
        if let Some(relationship_status) = profile.relationship_status {
            new_profile.relationship_status = Some(relationship_status);
        }
        if let Some(likes_attending_to) = profile.likes_attending_to {
            new_profile.likes_attending_to = Some(likes_attending_to);
        }
        if let Some(favorite_destinations) = profile.favorite_destinations {
            new_profile.favorite_destinations = Some(favorite_destinations);
        }
        if let Some(languages_spoken) = profile.languages_spoken {
            new_profile.languages_spoken = Some(languages_spoken);
        }
        if let Some(passions_and_hobbies) = profile.passions_and_hobbies {
            new_profile.passions_and_hobbies = Some(passions_and_hobbies);
        }

        partial.profile = Some(new_profile);
    }
    if let Some(temporary_password) = data.temporary_password {
        partial.temporary_password = Some(false);
    }
    user.update(db, partial, data.remove.unwrap_or_default())
        .await?;

    Ok(Json(user.foreign()))
}
