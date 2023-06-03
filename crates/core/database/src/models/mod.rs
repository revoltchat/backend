mod admin_migrations;
mod bots;
mod channel_webhooks;
mod files;
mod safety_strikes;
mod server_members;
mod servers;
mod user_settings;
mod users;

pub use admin_migrations::*;
pub use bots::*;
pub use channel_webhooks::*;
pub use files::*;
pub use safety_strikes::*;
pub use server_members::*;
pub use servers::*;
pub use user_settings::*;
pub use users::*;

use crate::{Database, MongoDb, ReferenceDb};

pub trait AbstractDatabase:
    Sync
    + Send
    + admin_migrations::AbstractMigrations
    + bots::AbstractBots
    + files::AbstractAttachments
    + safety_strikes::AbstractAccountStrikes
    + server_members::AbstractServerMembers
    + servers::AbstractServers
    + user_settings::AbstractUserSettings
    + users::AbstractUsers
    + channel_webhooks::AbstractWebhooks
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
