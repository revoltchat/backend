mod admin_migrations;
mod bots;

pub use admin_migrations::*;
pub use bots::*;

use crate::{Database, MongoDb, ReferenceDb};

pub trait AbstractDatabase:
    Sync + Send + admin_migrations::AbstractMigrations + bots::AbstractBots
{
}

impl AbstractDatabase for ReferenceDb {}
impl AbstractDatabase for MongoDb {}

impl std::ops::Deref for Database {
    type Target = dyn AbstractDatabase;

    fn deref(&self) -> &Self::Target {
        match &self {
            Database::Reference(dummy) => dummy,
            Database::MongoDb(mongo) => mongo,
        }
    }
}
