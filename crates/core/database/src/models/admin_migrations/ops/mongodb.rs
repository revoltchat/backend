use crate::MongoDb;

use super::AbstractMigrations;

mod init;
mod scripts;

#[async_trait]
impl AbstractMigrations for MongoDb {
    /// Migrate the database
    async fn migrate_database(&self) -> Result<(), ()> {
        info!("Migrating the database.");

        let list = self
            .list_database_names(None, None)
            .await
            .expect("Failed to fetch database names.");

        if list.iter().any(|x| x == "revolt") {
            scripts::migrate_database(self).await;
        } else {
            init::create_database(self).await;
        }

        Ok(())
    }
}
