bitfield! {
    pub struct MemberPermissions(MSB0 [u8]);
    u8;
    pub get_access, set_access: 7;
    pub get_create_invite, set_create_invite: 6;
    pub get_kick_members, set_kick_members: 5;
    pub get_ban_members, set_ban_members: 4;
    pub get_read_messages, set_read_messages: 3;
    pub get_send_messages, set_send_messages: 2;
}

use super::get_collection;
use crate::guards::channel::ChannelRef;
use crate::guards::guild::GuildRef;

use bson::{bson, doc};
use mongodb::options::FindOneOptions;

pub struct PermissionCalculator {
    pub user_id: String,
    pub channel: Option<ChannelRef>,
    pub guild: Option<GuildRef>,
}

impl PermissionCalculator {
    pub fn new(user_id: String) -> PermissionCalculator {
        PermissionCalculator {
            user_id,
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

    pub fn calculate(self) -> u8 {
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
                            "id": &self.user_id,
                        }
                    }
                },
                doc! { }
            ) {
                if guild.owner == self.user_id {
                    return u8::MAX;
                }

                permissions = guild.default_permissions;
            }
        }

        if let Some(channel) = &self.channel {
            match channel.channel_type {
                0 => {
                    if let Some(arr) = &channel.recipients {
                        for item in arr {
                            if item == &self.user_id {
                                permissions = 49;
                                break;
                            }
                        }
                    }
                },
                1 => {
                    unreachable!()
                },
                2 => {
                    // nothing implemented yet
                },
                _ => {}
            }
        }

        permissions as u8
    }

    pub fn as_permission(self) -> MemberPermissions<[u8; 1]> {
        MemberPermissions([ self.calculate() ])
    }
}
