use crate::ReferenceDb;

use super::AbstractMigrations;

#[async_trait]
impl AbstractMigrations for ReferenceDb {
    /// Migrate the database
    async fn migrate_database(&self) -> Result<(), ()> {
        // Here you would do your typical migrations if this was a real database.
        Ok(())
    }
}
