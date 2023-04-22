mod mongodb;
mod reference;

#[async_trait]
pub trait AbstractMigrations: Sync + Send {
    /// Migrate the database
    async fn migrate_database(&self) -> Result<(), ()>;
}
