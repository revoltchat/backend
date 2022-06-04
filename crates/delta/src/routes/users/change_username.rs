use crate::util::regex::RE_USERNAME;
use revolt_quark::{models::User, rauth::models::Account, Database, Error, Result};
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// # Username Information
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataChangeUsername {
    /// New username
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    username: String,
    /// Current account password
    #[validate(length(min = 8, max = 1024))]
    password: String,
}

/// # Change Username
///
/// Change your username.
#[openapi(tag = "User Information")]
#[patch("/@me/username", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    account: Account,
    mut user: User,
    data: Json<DataChangeUsername>,
) -> Result<Json<User>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    account
        .verify_password(&data.password)
        .map_err(|_| Error::InvalidCredentials)?;

    user.update_username(db, data.username).await?;
    Ok(Json(user.foreign()))
}
