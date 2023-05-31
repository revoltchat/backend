use revolt_quark::models::report::{ReportStatus, ReportStatusString, ReportedContent};
use revolt_quark::models::{Report, User};
use revolt_quark::{Db, Error, Result};
use rocket::serde::json::Json;
use serde::Deserialize;

/// # Query Parameters
#[derive(Deserialize, JsonSchema, FromForm)]
pub struct OptionsFetchReports {
    /// Find reports against messages, servers, or users
    content_id: Option<String>,

    /// Find reports created by user
    author_id: Option<String>,

    /// Report status to include in search
    status: Option<ReportStatusString>,
}

/// # Fetch Reports
///
/// Fetch all available reports
#[openapi(tag = "User Safety")]
#[get("/reports?<options..>")]
pub async fn fetch_reports(
    db: &Db,
    user: User,
    options: OptionsFetchReports,
) -> Result<Json<Vec<Report>>> {
    // Must be privileged for this route
    if !user.privileged {
        return Err(Error::NotPrivileged);
    }

    let mut reports = db.fetch_reports().await?;

    if let Some(content_id) = options.content_id {
        reports.retain(|report| match &report.content {
            ReportedContent::Message { id, .. }
            | ReportedContent::Server { id, .. }
            | ReportedContent::User { id, .. } => id == &content_id,
        });
    }

    if let Some(author_id) = options.author_id {
        reports.retain(|report| report.author_id == author_id);
    }

    if let Some(status) = options.status {
        reports.retain(|report| {
            matches!(
                (&status, &report.status),
                (ReportStatusString::Created, ReportStatus::Created { .. })
                    | (ReportStatusString::Rejected, ReportStatus::Rejected { .. })
                    | (ReportStatusString::Resolved, ReportStatus::Resolved { .. })
            )
        });
    }

    Ok(Json(reports))
}
