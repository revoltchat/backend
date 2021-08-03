use crate::database::*;

use rauth::auth::Session;
use rocket::serde::json::Value;

#[get("/hello")]
pub async fn req(_session: Session, user: Option<User>) -> Value {
    json!({
        "onboarding": user.is_none()
    })
}
