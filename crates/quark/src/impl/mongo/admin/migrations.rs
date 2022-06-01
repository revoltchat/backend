use crate::{AbstractMigrations, Result};

use super::super::MongoDb;

mod init;
mod scripts;

#[async_trait]
impl AbstractMigrations for MongoDb {
    async fn migrate_database(&self) -> Result<()> {
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
