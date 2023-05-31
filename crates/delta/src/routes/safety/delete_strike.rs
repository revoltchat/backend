use revolt_database::Database;
use revolt_quark::models::User;
use revolt_quark::{Error, Result};
use rocket::State;

/// # Delete Strike
///
/// Delete a strike by its ID
#[openapi(tag = "User Safety")]
#[delete("/strikes/<strike_id>")]
pub async fn delete_strike(db: &State<Database>, user: User, strike_id: String) -> Result<()> {
    // Must be privileged for this route
    if !user.privileged {
        return Err(Error::NotPrivileged);
    }

    let strike = db
        .fetch_account_strike(&strike_id)
        .await
        .map_err(Error::from_core)?;

    strike.delete(db).await.map_err(Error::from_core)
}
