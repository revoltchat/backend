use crate::ReferenceDb;

use super::AbstractMigrations;

#[async_trait]
impl AbstractMigrations for ReferenceDb {
    #[cfg(test)]
    /// Drop the database
    async fn drop_database(&self) {}

    /// Migrate the database
    async fn migrate_database(&self) -> Result<(), ()> {
        // Here you would do your typical migrations if this was a real database.
        Ok(())
    }
}
