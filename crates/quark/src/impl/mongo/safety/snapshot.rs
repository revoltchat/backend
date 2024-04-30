use crate::models::Snapshot;
use crate::{AbstractSnapshot, Result};

use super::super::MongoDb;

static COL: &str = "safety_snapshots";

#[async_trait]
impl AbstractSnapshot for MongoDb {
    async fn insert_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        self.insert_one(COL, snapshot).await.map(|_| ())
    }

    async fn fetch_snapshots(&self, report_id: &str) -> Result<Vec<Snapshot>> {
        self.find(
            COL,
            doc! {
                "report_id": report_id
            },
        )
        .await
    }
}
