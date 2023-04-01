use crate::{models::report::PartialReport, models::Report, Database, Result};

impl Report {
    /// Update report data
    pub async fn update(&mut self, db: &Database, partial: PartialReport) -> Result<()> {
        self.apply_options(partial.clone());
        db.update_report(&self.id, &partial).await
    }
}
