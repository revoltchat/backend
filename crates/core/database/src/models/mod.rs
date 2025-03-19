mod admin_migrations;
mod bots;
mod channel_webhooks;
mod channels;
mod events;
mod files;
mod ratelimit_events;
mod safety_strikes;
mod server_members;
mod servers;
pub mod trips;
mod user_settings;
mod user_white_list;
mod users;

pub use admin_migrations::*;
pub use bots::*;
pub use channel_webhooks::*;
pub use channels::*;
pub use events::*;
pub use files::*;
pub use ratelimit_events::*;
pub use safety_strikes::*;
pub use server_members::*;
pub use servers::*;
pub use trips::model::{Trip, TripComment};
pub use user_settings::*;
pub use user_white_list::*;
pub use users::*;

use crate::{Database, MongoDb, ReferenceDb};

pub trait AbstractDatabase:
    Sync
    + Send
    + admin_migrations::AbstractMigrations
    + bots::AbstractBots
    + channels::AbstractChannels
    + channel_webhooks::AbstractWebhooks
    + files::AbstractAttachments
    + ratelimit_events::AbstractRatelimitEvents
    + safety_strikes::AbstractAccountStrikes
    + server_members::AbstractServerMembers
    + servers::AbstractServers
    + user_settings::AbstractUserSettings
    + users::AbstractUsers
    + user_white_list::AbstractUserWhiteList
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
