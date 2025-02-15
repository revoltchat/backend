mod admin {
    pub mod stats;
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
    pub mod user_white_list;
}

mod safety {
    pub mod report;
    pub mod snapshot;
}

mod events {
    pub mod event;
}

pub use admin::stats::AbstractStats;

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
pub use users::user_white_list::AbstractUserWhiteList;

pub use safety::report::AbstractReport;
pub use safety::snapshot::AbstractSnapshot;

pub use events::event::AbstractEvents;
pub trait AbstractDatabase:
    Sync
    + Send
    + AbstractStats
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
    + AbstractReport
    + AbstractSnapshot
    + AbstractUserWhiteList
    + AbstractEvents
{
}
