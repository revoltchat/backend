use crate::{models::stats::Stats, AbstractStats, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractStats for DummyDb {
    async fn generate_stats(&self) -> Result<Stats> {
        todo!()
    }
}
