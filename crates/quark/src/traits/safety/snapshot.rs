use crate::models::Snapshot;
use crate::Result;

#[async_trait]
pub trait AbstractSnapshot: Sync + Send {
    /// Insert a new snapshot into the database
    async fn insert_snapshot(&self, snapshot: &Snapshot) -> Result<()>;

    /// Fetch a snapshot by a report's id
    async fn fetch_snapshot(&self, report_id: &str) -> Result<Snapshot>;
}
