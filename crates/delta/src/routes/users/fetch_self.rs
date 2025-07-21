use revolt_database::{User, util::oauth2::{OAuth2Scoped, scopes}};
use revolt_models::v0;
use revolt_result::Result;
use rocket::serde::json::Json;

/// # Fetch Self
///
/// Retrieve your user information.
#[openapi(tag = "User Information")]
#[get("/@me")]
pub async fn fetch_self(
    _oauth2_scope: OAuth2Scoped<scopes::ReadIdentify>,
    user: User
) -> Result<Json<v0::User>> {
    Ok(Json(user.into_self(false).await))
}
