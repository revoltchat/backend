use crate::models::Snapshot;
use crate::{AbstractSnapshot, Result};

use super::super::MongoDb;

static COL: &str = "safety_snapshots";

#[async_trait]
impl AbstractSnapshot for MongoDb {
    async fn insert_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        self.insert_one(COL, snapshot).await.map(|_| ())
    }
}
