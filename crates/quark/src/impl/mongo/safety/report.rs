use crate::models::Report;
use crate::{AbstractReport, Result};

use super::super::MongoDb;

static COL: &str = "safety_reports";

#[async_trait]
impl AbstractReport for MongoDb {
    async fn insert_report(&self, report: &Report) -> Result<()> {
        self.insert_one(COL, report).await.map(|_| ())
    }
}
