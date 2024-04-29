use authifier::models::Session;
use once_cell::sync::Lazy;
use regex::Regex;
use revolt_database::{Database, PartialUser, User};
use revolt_models::v0;
use revolt_result::{create_error, Result};

use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Regex for valid usernames
///
/// Block zero width space
/// Block lookalike characters
pub static RE_USERNAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\p{L}|[\d_.-])+$").unwrap());
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
}
/// # New User Data
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataOnboard {
    /// New username which will be used to identify the user on the platform
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
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
pub async fn complete(
    db: &State<Database>,
    session: Session,
    user: Option<User>,
    data: Json<DataOnboard>,
) -> Result<Json<v0::User>> {
    if user.is_some() {
        return Err(create_error!(AlreadyOnboarded));
    }

    let data: DataOnboard = data.into_inner();
    data.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;
    let mut partial: PartialUser = PartialUser {
        ..Default::default()
    };
    let profile = data.profile;
    let mut new_profile = partial.profile.take().unwrap_or_default();
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
    new_profile.occupation = Some(occupation);
    partial.profile = Some(new_profile);
    Ok(Json(
        User::create_onboarding(db, data.username, session.user_id, partial)
            .await?
            .into_self()
            .await,
    ))
}
