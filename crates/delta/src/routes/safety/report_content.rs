use revolt_quark::events::client::EventV1;
use revolt_quark::models::report::{ReportStatus, ReportedContent};
use revolt_quark::models::snapshot::{Snapshot, SnapshotContent};
use revolt_quark::models::{Report, User};
use revolt_quark::{Db, Error, Result};
use serde::Deserialize;
use ulid::Ulid;
use validator::Validate;

use rocket::serde::json::Json;

/// # Report Data
#[derive(Validate, Deserialize, JsonSchema)]
pub struct DataReportContent {
    /// Content being reported
    content: ReportedContent,
    /// Additional report description
    #[validate(length(min = 0, max = 1000))]
    #[serde(default)]
    additional_context: String,
}

/// # Report Content
///
/// Report a piece of content to the moderation team.
#[openapi(tag = "User Safety")]
#[post("/report", data = "<data>")]
pub async fn report_content(db: &Db, user: User, data: Json<DataReportContent>) -> Result<()> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    // Bots cannot create reports
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    // Find the content and create a snapshot of it
    // Also retrieve any references to Files
    let (snapshots, files): (Vec<SnapshotContent>, Vec<String>) = match &data.content {
        ReportedContent::Message { id, .. } => {
            let message = db.fetch_message(id).await?;

            // Users cannot report themselves
            if message.author == user.id {
                return Err(Error::CannotReportYourself);
            }

            let (snapshot, files) = SnapshotContent::generate_from_message(db, message).await?;
            (vec![snapshot], files)
        }
        ReportedContent::Server { id, .. } => {
            let server = db.fetch_server(id).await?;

            // Users cannot report their own server
            if server.owner == user.id {
                return Err(Error::CannotReportYourself);
            }

            let (snapshot, files) = SnapshotContent::generate_from_server(server)?;
            (vec![snapshot], files)
        }
        ReportedContent::User { id, message_id, .. } => {
            let reported_user = db.fetch_user(id).await?;

            // Users cannot report themselves
            if reported_user.id == user.id {
                return Err(Error::CannotReportYourself);
            }

            // Determine if there is a message provided as context
            let message = if let Some(id) = message_id {
                db.fetch_message(id).await.ok()
            } else {
                None
            };

            let (snapshot, files) = SnapshotContent::generate_from_user(reported_user)?;

            if let Some(message) = message {
                let (message_snapshot, message_files) =
                    SnapshotContent::generate_from_message(db, message).await?;
                (
                    vec![snapshot, message_snapshot],
                    [files, message_files].concat(),
                )
            } else {
                (vec![snapshot], files)
            }
        }
    };

    // Mark all the attachments as reported
    for file in files {
        db.mark_attachment_as_reported(&file).await?;
    }

    // Generate an id for the report
    let id = Ulid::new().to_string();

    // Insert all new generated snapshots
    for content in snapshots {
        // Save a snapshot of the content
        let snapshot = Snapshot {
            id: Ulid::new().to_string(),
            report_id: id.to_string(),
            content,
        };

        db.insert_snapshot(&snapshot).await?;
    }

    // Save the report
    let report = Report {
        id,
        author_id: user.id,
        content: data.content,
        additional_context: data.additional_context,
        status: ReportStatus::Created {},
        notes: String::new(),
    };

    db.insert_report(&report).await?;

    EventV1::ReportCreate(report).global().await;

    Ok(())
}
