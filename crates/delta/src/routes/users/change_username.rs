use authifier::models::Account;
use revolt_database::{Database, User};
use revolt_models::v0;
use revolt_result::{create_error, Result};
use crate::util::json::{Json, Validate};
use rocket::State;

/// # Change Username
///
/// Change your username.
#[openapi(tag = "User Information")]
#[patch("/@me/username", data = "<data>")]
pub async fn change_username(
    db: &State<Database>,
    account: Account,
    mut user: User,
    data: Validate<Json<v0::DataChangeUsername>>,
) -> Result<Json<v0::User>> {
    let data = data.into_inner().into_inner();

    account
        .verify_password(&data.password)
        .map_err(|_| create_error!(InvalidCredentials))?;

    user.update_username(db, data.username).await?;
    Ok(Json(user.into(db, None).await))
}
