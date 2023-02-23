use crate::models::Report;
use crate::Result;

#[async_trait]
pub trait AbstractReport: Sync + Send {
    /// Insert a new report into the database
    async fn insert_report(&self, report: &Report) -> Result<()>;
}
