use crate::util::result::{Error, Result};

use super::PermissionCalculator;
use super::Ref;

use num_enum::TryFromPrimitive;
use std::ops;

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Copy, Clone)]
#[repr(u32)]
pub enum ServerPermission {
    View = 0b00000000000000000000000000000001,           // 1
    ManageRoles = 0b00000000000000000000000000000010,    // 2
    ManageChannels = 0b00000000000000000000000000000100, // 4
    ManageServer = 0b00000000000000000000000000001000,   // 8
    KickMembers = 0b00000000000000000000000000010000,    // 16
    BanMembers = 0b00000000000000000000000000100000,     // 32
    // 6 bits of space
    ChangeNickname = 0b00000000000000000001000000000000, // 4096
    ManageNicknames = 0b00000000000000000010000000000000, // 8192
    ChangeAvatar = 0b00000000000000000100000000000000,   // 16382
    RemoveAvatars = 0b00000000000000001000000000000000,  // 32768
                                                         // 16 bits of space
}

lazy_static! {
    pub static ref DEFAULT_PERMISSION: u32 =
        ServerPermission::View
        + ServerPermission::ChangeNickname
        + ServerPermission::ChangeAvatar;
}

impl_op_ex!(+ |a: &ServerPermission, b: &ServerPermission| -> u32 { *a as u32 | *b as u32 });
impl_op_ex_commutative!(+ |a: &u32, b: &ServerPermission| -> u32 { *a | *b as u32 });

bitfield! {
    pub struct ServerPermissions(MSB0 [u32]);
    u32;
    pub get_view, _: 31;
    pub get_manage_roles, _: 30;
    pub get_manage_channels, _: 29;
    pub get_manage_server, _: 28;
    pub get_kick_members, _: 27;
    pub get_ban_members, _: 26;

    pub get_change_nickname, _: 19;
    pub get_manage_nicknames, _: 18;
    pub get_change_avatar, _: 17;
    pub get_remove_avatars, _: 16;
}

impl<'a> PermissionCalculator<'a> {
    pub async fn calculate_server(self) -> Result<u32> {
        let server = if let Some(server) = self.server {
            server
        } else {
            unreachable!()
        };

        if self.perspective.id == server.owner {
            Ok(u32::MAX)
        } else {
            match Ref::from_unchecked(self.perspective.id.clone()).fetch_member(&server.id).await {
                Ok(member) => {
                    let mut perm = server.default_permissions.0 as u32;
                    if let Some(roles) = member.roles {
                        for role in roles {
                            if let Some(server_role) = server.roles.get(&role) {
                                perm |= server_role.permissions.0 as u32;
                            }
                        }
                    }

                    Ok(perm)
                }
                Err(error) => {
                    match &error {
                        Error::NotFound => Ok(0),
                        _ => Err(error)
                    }
                }
            }
        }
    }

    pub async fn for_server(self) -> Result<ServerPermissions<[u32; 1]>> {
        Ok(ServerPermissions([self.calculate_server().await?]))
    }
}
