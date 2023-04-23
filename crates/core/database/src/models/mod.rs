mod admin_migrations;
mod bots;
mod files;
mod users;

pub use admin_migrations::*;
pub use bots::*;
pub use files::*;
pub use users::*;

use crate::{Database, MongoDb, ReferenceDb};

pub trait AbstractDatabase:
    Sync + Send + admin_migrations::AbstractMigrations + bots::AbstractBots + users::AbstractUsers
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
