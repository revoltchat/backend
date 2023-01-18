use revolt_quark::models::User;
use revolt_quark::{Db, Error, Result};
use serde::Deserialize;
use validator::Validate;

use rocket::serde::json::Json;

#[derive(Deserialize, JsonSchema)]
enum UserReportReason {
    NoneSpecified,
    InappropriateProfile,
}

// TODO: move me into models
#[derive(Deserialize, JsonSchema)]
pub enum ReportedContent {
    User {
        id: String,
        report_reason: UserReportReason,
    },
}

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

    // find the content and create the report here

    Ok(())
}
