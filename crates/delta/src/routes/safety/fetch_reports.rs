use revolt_quark::models::{Report, User};
use revolt_quark::{Db, Error, Result};
use rocket::serde::json::Json;

/// # Fetch Reports
///
/// Fetch all available reports
#[openapi(tag = "User Safety")]
#[get("/reports")]
pub async fn fetch_reports(db: &Db, user: User) -> Result<Json<Vec<Report>>> {
    // Must be privileged for this route
    if !user.privileged {
        return Err(Error::NotPrivileged);
    }

    db.fetch_reports().await.map(Json)
}
