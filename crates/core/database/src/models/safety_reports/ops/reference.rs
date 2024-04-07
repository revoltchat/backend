use revolt_result::Result;

use crate::ReferenceDb;
use crate::Report;

use super::AbstractReport;

#[async_trait]
impl AbstractReport for ReferenceDb {
    /// Insert a new report into the database
    async fn insert_report(&self, report: &Report) -> Result<()> {
        let mut reports = self.safety_reports.lock().await;
        if reports.contains_key(&report.id) {
            Err(create_database_error!("insert", "report"))
        } else {
            reports.insert(report.id.to_string(), report.clone());
            Ok(())
        }
    }
}
