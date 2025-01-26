use revolt_database::User;
use revolt_models::v0;
use revolt_result::Result;
use rocket::serde::json::Json;

/// # Fetch Self
///
/// Retrieve your user information.
#[openapi(tag = "User Information")]
#[get("/@me")]
pub async fn fetch(user: User) -> Result<Json<v0::User>> {
    Ok(Json(user.into_self(false).await))
}
