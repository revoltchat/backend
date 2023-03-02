use crate::models::report::PartialReport;
use crate::models::Report;
use crate::{AbstractReport, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractReport for DummyDb {
    async fn insert_report(&self, report: &Report) -> Result<()> {
        info!("Insert {:?}", report);
        Ok(())
    }

    async fn update_report(&self, _id: &str, _report: &PartialReport) -> Result<()> {
        todo!()
    }

    async fn fetch_report(&self, _report_id: &str) -> Result<Report> {
        todo!()
    }

    async fn fetch_reports(&self) -> Result<Vec<Report>> {
        Ok(vec![])
    }
}
