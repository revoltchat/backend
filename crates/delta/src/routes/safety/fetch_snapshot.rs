use revolt_quark::models::{Snapshot, User};
use revolt_quark::{Db, Error, Result};
use rocket::serde::json::Json;

/// # Fetch Snapshot
///
/// Fetch a snapshot for a given report
#[openapi(tag = "User Safety")]
#[get("/snapshot/<report_id>")]
pub async fn fetch_snapshot(db: &Db, user: User, report_id: String) -> Result<Json<Snapshot>> {
    // Must be privileged for this route
    if !user.privileged {
        return Err(Error::NotPrivileged);
    }

    db.fetch_snapshot(&report_id).await.map(Json)
}
