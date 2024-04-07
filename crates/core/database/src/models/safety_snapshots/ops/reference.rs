use revolt_result::Result;

use crate::ReferenceDb;
use crate::Snapshot;

use super::AbstractSnapshot;

#[async_trait]
impl AbstractSnapshot for ReferenceDb {
    /// Insert a new report into the database
    async fn insert_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        let mut snapshots = self.safety_snapshots.lock().await;
        if snapshots.contains_key(&snapshot.id) {
            Err(create_database_error!("insert", "snapshot"))
        } else {
            snapshots.insert(snapshot.id.to_string(), snapshot.clone());
            Ok(())
        }
    }
}
