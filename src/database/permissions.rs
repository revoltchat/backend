use super::mutual::has_mutual_connection;
use crate::database::user::UserRelationship;
use crate::guards::auth::UserRef;
use crate::guards::channel::ChannelRef;
use crate::guards::guild::GuildRef;

use bson::{bson, doc};
use num_enum::TryFromPrimitive;

#[derive(Debug, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum Relationship {
    FRIEND = 0,
    OUTGOING = 1,
    INCOMING = 2,
    BLOCKED = 3,
    BLOCKEDOTHER = 4,
    NONE = 5,
    SELF = 6,
}

#[derive(Debug, PartialEq, Eq, TryFromPrimitive)]
#[repr(u32)]
pub enum Permission {
    ACCESS = 1,
    CREATE_INVITE = 2,
    KICK_MEMBERS = 4,
    BAN_MEMBERS = 8,
    READ_MESSAGES = 16,
    SEND_MESSAGES = 32,
    MANAGE_MESSAGES = 64,
    MANAGE_CHANNELS = 128,
    MANAGE_SERVER = 256,
    MANAGE_ROLES = 512,
}

bitfield! {
    pub struct MemberPermissions(MSB0 [u32]);
    u8;
    pub get_access, set_access: 31;
    pub get_create_invite, set_create_invite: 30;
    pub get_kick_members, set_kick_members: 29;
    pub get_ban_members, set_ban_members: 28;
    pub get_read_messages, set_read_messages: 27;
    pub get_send_messages, set_send_messages: 26;
    pub get_manage_messages, set_manage_messages: 25;
    pub get_manage_channels, set_manage_channels: 24;
    pub get_manage_server, set_manage_server: 23;
    pub get_manage_roles, set_manage_roles: 22;
}

pub fn get_relationship_internal(
    user_id: &str,
    target_id: &str,
    relationships: &Option<Vec<UserRelationship>>,
) -> Relationship {
    if user_id == target_id {
        return Relationship::SELF;
    }

    if let Some(arr) = &relationships {
        for entry in arr {
            if entry.id == target_id {
                match entry.status {
                    0 => return Relationship::FRIEND,
                    1 => return Relationship::OUTGOING,
                    2 => return Relationship::INCOMING,
                    3 => return Relationship::BLOCKED,
                    4 => return Relationship::BLOCKEDOTHER,
                    _ => return Relationship::NONE,
                }
            }
        }
    }

    Relationship::NONE
}

pub fn get_relationship(a: &UserRef, b: &UserRef) -> Relationship {
    if a.id == b.id {
        return Relationship::SELF;
    }

    get_relationship_internal(&a.id, &b.id, &a.fetch_relationships())
}

pub struct PermissionCalculator {
    pub user: UserRef,
    pub channel: Option<ChannelRef>,
    pub guild: Option<GuildRef>,
}

impl PermissionCalculator {
    pub fn new(user: UserRef) -> PermissionCalculator {
        PermissionCalculator {
            user,
            channel: None,
            guild: None,
        }
    }

    pub fn channel(self, channel: ChannelRef) -> PermissionCalculator {
        PermissionCalculator {
            channel: Some(channel),
            ..self
        }
    }

    pub fn guild(self, guild: GuildRef) -> PermissionCalculator {
        PermissionCalculator {
            guild: Some(guild),
            ..self
        }
    }

    pub fn calculate(self) -> u32 {
        let guild = if let Some(value) = self.guild {
            Some(value)
        } else if let Some(channel) = &self.channel {
            match channel.channel_type {
                0..=1 => None,
                2 => {
                    if let Some(id) = &channel.guild {
                        GuildRef::from(id.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        };

        let mut permissions = 0;
        if let Some(guild) = guild {
            if let Some(_data) = guild.fetch_data_given(
                doc! {
                    "members": {
                        "$elemMatch": {
                            "id": &self.user.id,
                        }
                    }
                },
                doc! {},
            ) {
                if guild.owner == self.user.id {
                    return u32::MAX;
                }

                permissions = guild.default_permissions;
            }
        }

        if let Some(channel) = &self.channel {
            match channel.channel_type {
                0 => {
                    // ? check user is part of the channel
                    if let Some(arr) = &channel.recipients {
                        let mut other_user = "";
                        for item in arr {
                            if item == &self.user.id {
                                permissions = 49;
                            } else {
                                other_user = item;
                            }
                        }

                        let relationships = self.user.fetch_relationships();
                        let relationship =
                            get_relationship_internal(&self.user.id, &other_user, &relationships);

                        if relationship == Relationship::BLOCKED
                            || relationship == Relationship::BLOCKEDOTHER
                        {
                            permissions = 1;
                        } else if has_mutual_connection(self.user.id, other_user.to_string()) {
                            permissions = 49;
                        }
                    }
                }
                1 => unreachable!(),
                2 => {
                    // nothing implemented yet
                }
                _ => {}
            }
        }

        permissions as u32
    }

    pub fn as_permission(self) -> MemberPermissions<[u32; 1]> {
        MemberPermissions([self.calculate()])
    }
}
