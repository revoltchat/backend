use crate::database::*;

use rauth::auth::Session;
use rocket_contrib::json::JsonValue;

#[get("/hello")]
pub async fn req(_session: Session, user: Option<User>) -> JsonValue {
    json!({
        "onboarding": user.is_none()
    })
}
