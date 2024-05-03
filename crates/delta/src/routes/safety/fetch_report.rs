use revolt_quark::models::{Report, User};
use revolt_quark::{Db, Error, Result};
use rocket::serde::json::Json;

/// # Fetch Report
///
/// Fetch a report by its ID
#[openapi(tag = "User Safety")]
#[get("/report/<id>")]
pub async fn fetch_report(db: &Db, user: User, id: String) -> Result<Json<Report>> {
    // Must be privileged for this route
    if !user.privileged {
        return Err(Error::NotPrivileged);
    }

    db.fetch_report(&id).await.map(Json)
}
