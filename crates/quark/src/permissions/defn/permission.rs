use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use std::ops;

/// Permission value on Revolt
///
/// This should be restricted to the lower 52 bits to prevent any
/// potential issues with Javascript. Also leave empty spaces for
/// future permission flags to be added.
#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, TryFromPrimitive, Copy, Clone, JsonSchema,
)]
#[repr(u64)]
pub enum Permission {
    // * Generic permissions
    /// Manage the channel or channels on the server
    ManageChannel = 1 << 0,
    /// Manage the server
    ManageServer = 1 << 1,
    /// Manage permissions on servers or channels
    ManagePermissions = 1 << 2,
    /// Manage roles on server
    ManageRole = 1 << 3,
    /// Manage server customisation (includes emoji)
    ManageCustomisation = 1 << 4,

    // % 1 bits reserved

    // * Member permissions
    /// Kick other members below their ranking
    KickMembers = 1 << 6,
    /// Ban other members below their ranking
    BanMembers = 1 << 7,
    /// Timeout other members below their ranking
    TimeoutMembers = 1 << 8,
    /// Assign roles to members below their ranking
    AssignRoles = 1 << 9,
    /// Change own nickname
    ChangeNickname = 1 << 10,
    /// Change or remove other's nicknames below their ranking
    ManageNicknames = 1 << 11,
    /// Change own avatar
    ChangeAvatar = 1 << 12,
    /// Remove other's avatars below their ranking
    RemoveAvatars = 1 << 13,

    // % 7 bits reserved

    // * Channel permissions
    /// View a channel
    ViewChannel = 1 << 20,
    /// Read a channel's past message history
    ReadMessageHistory = 1 << 21,
    /// Send a message in a channel
    SendMessage = 1 << 22,
    /// Delete messages in a channel
    ManageMessages = 1 << 23,
    /// Manage webhook entries on a channel
    ManageWebhooks = 1 << 24,
    /// Create invites to this channel
    InviteOthers = 1 << 25,
    /// Send embedded content in this channel
    SendEmbeds = 1 << 26,
    /// Send attachments and media in this channel
    UploadFiles = 1 << 27,
    /// Masquerade messages using custom nickname and avatar
    Masquerade = 1 << 28,
    /// React to messages with emojis
    React = 1 << 29,

    // * Voice permissions
    /// Connect to a voice channel
    Connect = 1 << 30,
    /// Speak in a voice call
    Speak = 1 << 31,
    /// Share video in a voice call
    Video = 1 << 32,
    /// Mute other members with lower ranking in a voice call
    MuteMembers = 1 << 33,
    /// Deafen other members with lower ranking in a voice call
    DeafenMembers = 1 << 34,
    /// Move members between voice channels
    MoveMembers = 1 << 35,

    // * Misc. permissions
    // % Bits 36 to 52: free area
    // % Bits 53 to 64: do not use

    // * Grant all permissions
    /// Safely grant all permissions
    GrantAllSafe = 0x000F_FFFF_FFFF_FFFF,

    /// Grant all permissions
    GrantAll = u64::MAX,
}

impl_op_ex!(+ |a: &Permission, b: &Permission| -> u64 { *a as u64 | *b as u64 });
impl_op_ex_commutative!(+ |a: &u64, b: &Permission| -> u64 { *a | *b as u64 });

lazy_static! {
    pub static ref ALLOW_IN_TIMEOUT: u64 = Permission::ViewChannel + Permission::ReadMessageHistory;
    pub static ref DEFAULT_PERMISSION_VIEW_ONLY: u64 =
        Permission::ViewChannel + Permission::ReadMessageHistory;
    pub static ref DEFAULT_PERMISSION: u64 = *DEFAULT_PERMISSION_VIEW_ONLY
        + Permission::SendMessage
        + Permission::InviteOthers
        + Permission::SendEmbeds
        + Permission::UploadFiles
        + Permission::Connect
        + Permission::Speak;
    pub static ref DEFAULT_PERMISSION_SAVED_MESSAGES: u64 = Permission::GrantAllSafe as u64;
    pub static ref DEFAULT_PERMISSION_DIRECT_MESSAGE: u64 =
        *DEFAULT_PERMISSION + Permission::ManageChannel;
    pub static ref DEFAULT_PERMISSION_SERVER: u64 =
        *DEFAULT_PERMISSION + Permission::ChangeNickname + Permission::ChangeAvatar;
}

bitfield! {
    #[derive(Default)]
    pub struct Permissions(MSB0 [u64]);
    u64;

    // * Server permissions
    pub can_manage_channel, _: 63;
    pub can_manage_server, _: 62;
    pub can_manage_permissions, _: 61;
    pub can_manage_roles, _: 60;
    pub can_manage_customisation, _: 59;

    // * Member permissions
    pub can_kick_members, _: 57;
    pub can_ban_members, _: 56;
    pub can_timeout_members, _: 55;
    pub can_assign_roles, _: 54;
    pub can_change_nickname, _: 53;
    pub can_manage_nicknames, _: 52;
    pub can_change_avatar, _: 51;
    pub can_remove_avatars, _: 50;

    // * Channel permissions
    pub can_view_channel, _: 42;
    pub can_read_message_history, _: 41;
    pub can_send_message, _: 40;
    pub can_manage_messages, _: 39;
    pub can_manage_webhooks, _: 38;
    pub can_invite_others, _: 37;
    pub can_send_embeds, _: 36;
    pub can_upload_files, _: 35;
    pub can_masquerade, _: 34;

    // * Voice permissions
    pub can_connect, _: 32;
    pub can_speak, _: 31;
    pub can_share_video, _: 30;
    pub can_mute_members, _: 29;
    pub can_deafen_members, _: 28;
    pub can_move_members, _: 27;
}

pub type Perms = Permissions<[u64; 1]>;
