use crate::models::report::PartialReport;
use crate::models::Report;
use crate::{AbstractReport, Result};

use super::super::MongoDb;

static COL: &str = "safety_reports";

#[async_trait]
impl AbstractReport for MongoDb {
    async fn insert_report(&self, report: &Report) -> Result<()> {
        self.insert_one(COL, report).await.map(|_| ())
    }

    async fn update_report(&self, id: &str, report: &PartialReport) -> Result<()> {
        self.update_one_by_id(COL, id, report, vec![], None)
            .await
            .map(|_| ())
    }

    async fn fetch_report(&self, report_id: &str) -> Result<Report> {
        self.find_one_by_id(COL, report_id).await
    }

    async fn fetch_reports(&self) -> Result<Vec<Report>> {
        self.find(COL, doc! {}).await
    }
}
