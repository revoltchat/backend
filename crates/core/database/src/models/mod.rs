mod admin_migrations;
mod bots;
mod channel_invites;
mod channel_webhooks;
mod channels;
mod files;
mod ratelimit_events;
mod server_members;
mod servers;
mod user_settings;
mod users;

pub use admin_migrations::*;
pub use bots::*;
pub use channel_invites::*;
pub use channel_webhooks::*;
pub use channels::*;
pub use files::*;
pub use ratelimit_events::*;
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
    + channels::AbstractChannels
    + channel_invites::AbstractChannelInvites
    + channel_webhooks::AbstractWebhooks
    + files::AbstractAttachments
    + ratelimit_events::AbstractRatelimitEvents
    + server_members::AbstractServerMembers
    + servers::AbstractServers
    + user_settings::AbstractUserSettings
    + users::AbstractUsers
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
