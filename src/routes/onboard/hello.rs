use rocket_contrib::json::JsonValue;
use crate::database::entities::User;
use rauth::auth::Session;

#[get("/hello")]
pub async fn req(_session: Session, user: Option<User>) -> JsonValue {
    json!({
        "onboarding": user.is_none()
    })
}
