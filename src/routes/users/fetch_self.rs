use revolt_quark::Result;
use revolt_quark::models::User;

use rocket::serde::json::Json;

#[get("/@me")]
pub async fn req(user: User) -> Result<Json<User>> {
    Ok(Json(user))
}
