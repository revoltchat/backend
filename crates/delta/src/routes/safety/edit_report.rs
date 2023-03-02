use revolt_quark::{
    models::{
        report::{PartialReport, ReportStatus},
        Report, User,
    },
    Db, Error, Ref, Result,
};
use rocket::serde::json::Json;
use serde::Deserialize;
use validator::Validate;

/// # Report Data
#[derive(Validate, Deserialize, JsonSchema)]
pub struct DataEditReport {
    /// New report status
    status: Option<ReportStatus>,
    /// Report notes
    notes: Option<String>,
}

/// # Edit Report
///
/// Edit a report.
#[openapi(tag = "User Safety")]
#[patch("/reports/<report>", data = "<edit>")]
pub async fn edit_report(
    db: &Db,
    user: User,
    report: Ref,
    edit: Json<DataEditReport>,
) -> Result<Json<Report>> {
    // Must be privileged for this route
    if !user.privileged {
        return Err(Error::NotPrivileged);
    }

    // Validate data
    let edit = edit.into_inner();
    edit.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    // Create and apply update to report
    let mut report = report.as_report(db).await?;
    report
        .update(
            db,
            PartialReport {
                status: edit.status,
                notes: edit.notes,
                ..Default::default()
            },
        )
        .await?;

    Ok(Json(report))
}
