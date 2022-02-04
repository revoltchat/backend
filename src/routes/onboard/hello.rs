use rauth::entities::Session;
use revolt_quark::models::User;
use rocket::serde::json::Value;

#[get("/hello")]
pub async fn req(_session: Session, user: Option<User>) -> Value {
    json!({
        "onboarding": user.is_none()
    })
}
