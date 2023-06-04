use revolt_database::Database;
use revolt_models::v0::AccountStrike;
use revolt_quark::models::User;
use revolt_quark::{Error, Result};
use rocket::serde::json::Json;
use rocket::State;

/// # Fetch Strikes
///
/// Fetch strikes for a user by their ID
#[openapi(tag = "User Safety")]
#[get("/strikes/<user_id>")]
pub async fn fetch_strikes(
    db: &State<Database>,
    user: User,
    user_id: String,
) -> Result<Json<Vec<AccountStrike>>> {
    // Must be privileged for this route
    if !user.privileged {
        return Err(Error::NotPrivileged);
    }

    db.fetch_account_strikes_by_user(&user_id)
        .await
        .map(|v| v.into_iter().map(|e| e.into()).collect())
        .map(Json)
        .map_err(Error::from_core)
}
