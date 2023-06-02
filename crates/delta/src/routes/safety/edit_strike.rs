use revolt_database::{Database, PartialAccountStrike};
use revolt_models::v0::DataEditAccountStrike;
use revolt_quark::models::User;
use revolt_quark::{Error, Result};
use rocket::serde::json::Json;
use rocket::State;

/// # Edit Strike
///
/// Edit a strike by its ID
#[openapi(tag = "User Safety")]
#[post("/strikes/<strike_id>", data = "<data>")]
pub async fn edit_strike(
    db: &State<Database>,
    user: User,
    strike_id: String,
    data: Json<DataEditAccountStrike>,
) -> Result<()> {
    // Must be privileged for this route
    if !user.privileged {
        return Err(Error::NotPrivileged);
    }

    let mut strike = db
        .fetch_account_strike(&strike_id)
        .await
        .map_err(Error::from_core)?;

    strike
        .update(
            db,
            PartialAccountStrike {
                reason: Some(data.0.reason),
                ..Default::default()
            },
        )
        .await
        .map_err(Error::from_core)
}
