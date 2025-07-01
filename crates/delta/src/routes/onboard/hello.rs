use authifier::models::Session;
use revolt_database::User;
use revolt_models::v0;

use crate::util::json::Json;

/// # Check Onboarding Status
///
/// This will tell you whether the current account requires onboarding or whether you can continue to send requests as usual. You may skip calling this if you're restoring an existing session.
#[openapi(tag = "Onboarding")]
#[get("/hello")]
pub async fn hello(_session: Session, user: Option<User>) -> Json<v0::DataHello> {
    Json(v0::DataHello {
        onboarding: user.is_none(),
    })
}
