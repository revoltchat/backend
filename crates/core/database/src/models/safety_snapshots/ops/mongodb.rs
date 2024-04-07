use revolt_result::Result;

use crate::MongoDb;
use crate::Snapshot;

use super::AbstractSnapshot;

static COL: &str = "safety_snapshots";

#[async_trait]
impl AbstractSnapshot for MongoDb {
    /// Insert a new snapshot into the database
    async fn insert_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        query!(self, insert_one, COL, &snapshot).map(|_| ())
    }
}
