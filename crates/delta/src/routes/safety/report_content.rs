use revolt_quark::models::report::ReportedContent;
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
    let (content, files): (SnapshotContent, Vec<String>) = match &data.content {
        ReportedContent::Message { id, .. } => {
            let message = db.fetch_message(id).await?;

            // Collect message attachments
            let files = message
                .attachments
                .as_ref()
                .map(|attachments| attachments.iter().map(|x| x.id.to_string()).collect())
                .unwrap_or_default();

            (SnapshotContent::Message(message), files)
        }
        ReportedContent::Server { id, .. } => {
            let server = db.fetch_server(id).await?;

            // Collect server's icon and banner
            let files = [&server.icon, &server.banner]
                .iter()
                .filter_map(|x| x.as_ref().map(|x| x.id.to_string()))
                .collect();

            (SnapshotContent::Server(server), files)
        }
        ReportedContent::User { id, .. } => {
            let user = db.fetch_user(id).await?;

            // Collect user's avatar and profile background
            let files = [
                user.avatar.as_ref(),
                user.profile
                    .as_ref()
                    .and_then(|profile| profile.background.as_ref()),
            ]
            .iter()
            .filter_map(|x| x.as_ref().map(|x| x.id.to_string()))
            .collect();

            (SnapshotContent::User(user), files)
        }
    };

    // Mark all the attachments as reported
    for file in files {
        db.mark_attachment_as_reported(&file).await?;
    }

    // Generate an id for the report
    let id = Ulid::new().to_string();

    // Save a snapshot of the content
    let snapshot = Snapshot {
        id: Ulid::new().to_string(),
        report_id: id.to_string(),
        content,
    };

    db.insert_snapshot(&snapshot).await?;

    // Save the report
    let report = Report {
        id,
        author_id: user.id,
        content: data.content,
        additional_context: data.additional_context,
    };

    db.insert_report(&report).await?;

    Ok(())
}
