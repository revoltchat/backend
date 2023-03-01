use crate::models::Snapshot;
use crate::{AbstractSnapshot, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractSnapshot for DummyDb {
    async fn insert_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        info!("Insert {:?}", snapshot);
        Ok(())
    }

    async fn fetch_snapshot(&self, _report_id: &str) -> Result<Snapshot> {
        todo!()
    }
}
