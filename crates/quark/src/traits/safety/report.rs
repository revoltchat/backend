use crate::models::report::PartialReport;
use crate::models::Report;
use crate::Result;

#[async_trait]
pub trait AbstractReport: Sync + Send {
    /// Insert a new report into the database
    async fn insert_report(&self, report: &Report) -> Result<()>;

    /// Update a given report with new information
    async fn update_report(&self, id: &str, message: &PartialReport) -> Result<()>;

    /// Fetch report
    async fn fetch_report(&self, report_id: &str) -> Result<Report>;

    /// Fetch reports
    async fn fetch_reports(&self) -> Result<Vec<Report>>;
}
