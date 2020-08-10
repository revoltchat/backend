use super::mutual::has_mutual_connection;
use crate::database::channel::Channel;
use crate::database::guild::{fetch_guild, fetch_member, Guild, Member, MemberKey};
use crate::database::user::{User, UserRelationship};

use num_enum::TryFromPrimitive;

#[derive(Debug, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum Relationship {
    Friend = 0,
    Outgoing = 1,
    Incoming = 2,
    Blocked = 3,
    BlockedOther = 4,
    NONE = 5,
    SELF = 6,
}

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Copy, Clone)]
#[repr(u32)]
pub enum Permission {
    Access = 1,
    CreateInvite = 2,
    KickMembers = 4,
    BanMembers = 8,
    ReadMessages = 16,
    SendMessages = 32,
    ManageMessages = 64,
    ManageChannels = 128,
    ManageServer = 256,
    ManageRoles = 512,
    SendDirectMessages = 1024,
}

bitfield! {
    pub struct MemberPermissions(MSB0 [u32]);
    u32;
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
    pub get_send_direct_messages, set_send_direct_messages: 21;
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
                    0 => return Relationship::Friend,
                    1 => return Relationship::Outgoing,
                    2 => return Relationship::Incoming,
                    3 => return Relationship::Blocked,
                    4 => return Relationship::BlockedOther,
                    _ => return Relationship::NONE,
                }
            }
        }
    }

    Relationship::NONE
}

pub fn get_relationship(a: &User, b: &User) -> Relationship {
    if a.id == b.id {
        return Relationship::SELF;
    }

    get_relationship_internal(&a.id, &b.id, &a.relations)
}

pub struct PermissionCalculator {
    pub user: User,
    pub channel: Option<Channel>,
    pub guild: Option<Guild>,
    pub member: Option<Member>,
}

impl PermissionCalculator {
    pub fn new(user: User) -> PermissionCalculator {
        PermissionCalculator {
            user,
            channel: None,
            guild: None,
            member: None,
        }
    }

    pub fn channel(self, channel: Channel) -> PermissionCalculator {
        PermissionCalculator {
            channel: Some(channel),
            ..self
        }
    }

    pub fn guild(self, guild: Guild) -> PermissionCalculator {
        PermissionCalculator {
            guild: Some(guild),
            ..self
        }
    }

    pub fn fetch_data(mut self) -> PermissionCalculator {
        let guild = if let Some(value) = self.guild {
            Some(value)
        } else if let Some(channel) = &self.channel {
            match channel.channel_type {
                0..=1 => None,
                2 => {
                    if let Some(id) = &channel.guild {
                        if let Ok(result) = fetch_guild(id) {
                            result
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        };

        if let Some(guild) = &guild {
            if let Ok(result) = fetch_member(MemberKey(guild.id.clone(), self.user.id.clone())) {
                self.member = result;
            }
        }

        self.guild = guild;
        self
    }

    pub fn calculate(&self) -> u32 {
        let mut permissions: u32 = 0;
        if let Some(guild) = &self.guild {
            if let Some(_member) = &self.member {
                // ? logic should match mutual.rs#has_mutual_connection
                if guild.owner == self.user.id {
                    return u32::MAX;
                }

                permissions = guild.default_permissions as u32;
            }
        }

        if let Some(channel) = &self.channel {
            match channel.channel_type {
                0 => {
                    if let Some(arr) = &channel.recipients {
                        let mut other_user = None;
                        for item in arr {
                            if item != &self.user.id {
                                other_user = Some(item);
                            }
                        }

                        if let Some(other) = other_user {
                            let relationship =
                                get_relationship_internal(&self.user.id, &other, &self.user.relations);
    
                            if relationship == Relationship::Friend {
                                permissions = 1024 + 128 + 32 + 16 + 1;
                            } else if relationship == Relationship::Blocked
                                || relationship == Relationship::BlockedOther
                            {
                                permissions = 1;
                            } else if has_mutual_connection(&self.user.id, other, true) {
                                permissions = 1024 + 128 + 32 + 16 + 1;
                            } else {
                                permissions = 1;
                            }
                        } else {
                            // ? In this case, it is a "self DM".
                            return 1024 + 128 + 32 + 16 + 1;
                        }
                    }
                }
                1 => {
                    if let Some(id) = &channel.owner {
                        if &self.user.id == id {
                            return u32::MAX;
                        }
                    }

                    if let Some(arr) = &channel.recipients {
                        for item in arr {
                            if item == &self.user.id {
                                permissions = 177;
                                break;
                            }
                        }
                    }
                }
                2 => {
                    // nothing implemented yet
                }
                _ => {}
            }
        }

        permissions
    }

    pub fn as_permission(&self) -> MemberPermissions<[u32; 1]> {
        MemberPermissions([self.calculate()])
    }
}
