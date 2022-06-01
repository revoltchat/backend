use crate::{AbstractMigrations, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractMigrations for DummyDb {
    async fn migrate_database(&self) -> Result<()> {
        info!("Migrating the database.");
        Ok(())
    }
}
