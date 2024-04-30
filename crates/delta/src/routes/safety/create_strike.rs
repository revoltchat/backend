use revolt_database::{AccountStrike, Database};
use revolt_models::v0::{AccountStrike as AccountStrikeModel, DataCreateStrike};
use revolt_quark::models::User;
use revolt_quark::{Error, Result};
use rocket::serde::json::Json;
use rocket::State;

/// # Create Strike
///
/// Create a new account strike
#[openapi(tag = "User Safety")]
#[post("/strikes", data = "<data>")]
pub async fn create_strike(
    db: &State<Database>,
    user: User,
    data: Json<DataCreateStrike>,
) -> Result<Json<AccountStrikeModel>> {
    // Must be privileged for this route
    if !user.privileged {
        return Err(Error::NotPrivileged);
    }

    let data = data.into_inner();
    let target = db
        .fetch_user(&data.user_id)
        .await
        .map_err(Error::from_core)?;

    AccountStrike::create(db, target.id, data.reason, user.id)
        .await
        .map(|strike| strike.into())
        .map(Json)
        .map_err(Error::from_core)
}
