use crate::{models::stats::Stats, Result};

#[async_trait]
pub trait AbstractStats: Sync + Send {
    async fn generate_stats(&self) -> Result<Stats>;
}
