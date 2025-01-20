use revolt_quark::models::File;
use revolt_quark::{
    authifier::models::Session, models::User, Database, EmptyResponse, Error, Result,
};

use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// # New User Data
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataUserProfile {
    /// Text to set as user profile description
    #[validate(length(min = 1, max = 2000))]
    content: String,
    #[validate(length(min = 1, max = 2000))]
    first_name: String,
    /// Last name
    #[validate(length(min = 1, max = 2000))]
    last_name: String,
    /// Phone number
    #[validate(length(min = 1, max = 2000))]
    phone_number: String,
    /// Country
    #[validate(length(min = 1, max = 2000))]
    country: String,
    /// City
    #[validate(length(min = 1, max = 2000))]
    city: String,
    /// Occupation
    #[validate(length(min = 1, max = 2000))]
    occupation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facebook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instagram: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tik_tok: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub likes_attending_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favorite_destinations: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub languages_spoken: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passions_and_hobbies: Option<String>,
}
/// # New User Data
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataOnboard {
    /// New username which will be used to identify the user on the platform
    #[validate(length(min = 2))]
    username: String,
    #[validate(length(min = 2))]
    avatar: String,
    profile: DataUserProfile,
}

/// # Complete Onboarding
///
/// This sets a new username, completes onboarding and allows a user to start using Revolt.
#[openapi(tag = "Onboarding")]
#[post("/complete", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    session: Session,
    user: Option<User>,
    data: Json<DataOnboard>,
) -> Result<EmptyResponse> {
    if user.is_some() {
        return Err(Error::AlreadyOnboarded);
    }

    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let username = User::validate_username(data.username)?;
    let mut user = User {
        id: session.user_id,
        discriminator: User::find_discriminator(db, &username, None).await?,
        username,
        ..Default::default()
    };

    user.avatar = Some(File::use_avatar(db, &data.avatar, &user.id).await?);
    let profile = data.profile;
    let mut new_profile = user.profile.take().unwrap_or_default();
    let content = profile.content;
    new_profile.content = Some(content);
    let first_name = profile.first_name;
    new_profile.first_name = Some(first_name);
    let last_name = profile.last_name;
    new_profile.last_name = Some(last_name);
    let phone_number = profile.phone_number;
    new_profile.phone_number = Some(phone_number);
    let country = profile.country;
    new_profile.country = Some(country);
    let city = profile.city;
    new_profile.city = Some(city);
    let occupation = profile.occupation;
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
    new_profile.occupation = Some(occupation);
    user.profile = Some(new_profile);

    db.insert_user(&user).await.map(|_| EmptyResponse)
}
