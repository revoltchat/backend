use crate::util::regex::RE_USERNAME;
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
    new_profile.occupation = Some(occupation);
    user.profile = Some(new_profile);
    user.avatar = Some(File::use_avatar(db, &data.avatar, &user.id).await?);

    db.insert_user(&user).await.map(|_| EmptyResponse)
}
