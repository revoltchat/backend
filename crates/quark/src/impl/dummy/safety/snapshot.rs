use crate::models::Snapshot;
use crate::{AbstractSnapshot, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractSnapshot for DummyDb {
    async fn insert_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        info!("Insert {:?}", snapshot);
        Ok(())
    }

    async fn fetch_snapshots(&self, _report_id: &str) -> Result<Vec<Snapshot>> {
        todo!()
    }
}
