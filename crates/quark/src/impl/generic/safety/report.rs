use iso8601_timestamp::Timestamp;

use crate::{
    models::report::PartialReport,
    models::{report::ReportStatus, Report},
    Database, Result,
};

impl Report {
    /// Update report data
    pub async fn update(&mut self, db: &Database, partial: PartialReport) -> Result<()> {
        self.apply_options(partial.clone());

        match &mut self.status {
            ReportStatus::Created {} => {}
            ReportStatus::Rejected { closed_at, .. } => {
                if closed_at.is_none() {
                    closed_at.replace(Timestamp::now_utc());
                }
            }
            ReportStatus::Resolved { closed_at } => {
                if closed_at.is_none() {
                    closed_at.replace(Timestamp::now_utc());
                }
            }
        }

        db.update_report(&self.id, &partial).await
    }
}
