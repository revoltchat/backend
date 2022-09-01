mod admin {
    pub mod migrations;
}

mod media {
    pub mod attachment;
    pub mod emoji;
}

mod channels {
    pub mod channel;
    pub mod channel_invite;
    pub mod channel_unread;
    pub mod message;
}

mod servers {
    pub mod server;
    pub mod server_ban;
    pub mod server_member;
}

mod users {
    pub mod bot;
    pub mod user;
    pub mod user_settings;
}

pub use admin::migrations::AbstractMigrations;

pub use media::attachment::AbstractAttachment;
pub use media::emoji::AbstractEmoji;

pub use channels::channel::AbstractChannel;
pub use channels::channel_invite::AbstractChannelInvite;
pub use channels::channel_unread::AbstractChannelUnread;
pub use channels::message::AbstractMessage;

pub use servers::server::AbstractServer;
pub use servers::server_ban::AbstractServerBan;
pub use servers::server_member::AbstractServerMember;

pub use users::bot::AbstractBot;
pub use users::user::AbstractUser;
pub use users::user_settings::AbstractUserSettings;

pub trait AbstractDatabase:
    Sync
    + Send
    + AbstractMigrations
    + AbstractAttachment
    + AbstractEmoji
    + AbstractChannel
    + AbstractChannelInvite
    + AbstractChannelUnread
    + AbstractMessage
    + AbstractServer
    + AbstractServerBan
    + AbstractServerMember
    + AbstractBot
    + AbstractUser
    + AbstractUserSettings
{
}
