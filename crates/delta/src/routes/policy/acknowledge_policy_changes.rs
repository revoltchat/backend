use revolt_database::{events::client::EventV1, Database, Report, Snapshot, SnapshotContent, User};
use revolt_models::v0::{ReportStatus, ReportedContent};
use revolt_result::{create_error, Result};
use serde::Deserialize;
use ulid::Ulid;
use validator::Validate;

use rocket::{serde::json::Json, State};

/// # Acknowledge Policy Changes
///
/// Accept/acknowledge changes to platform policy.
#[openapi(tag = "Policy")]
#[post("/acknowledge")]
pub async fn acknowledge_policy_changes(db: &State<Database>, user: User) -> Result<()> {
    db.acknowledge_policy_changes(&user.id).await
}
