use crate::models::Report;
use crate::{AbstractReport, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractReport for DummyDb {
    async fn insert_report(&self, report: &Report) -> Result<()> {
        info!("Insert {:?}", report);
        Ok(())
    }
}
