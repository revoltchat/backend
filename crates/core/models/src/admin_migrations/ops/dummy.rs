use revolt_database::DummyDb;

use super::AbstractMigrations;

#[async_trait]
impl AbstractMigrations for DummyDb {
    /// Migrate the database
    async fn migrate_database(&self) -> Result<(), ()> {
        Ok(())
    }
}
