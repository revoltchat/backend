use crate::database::*;
use crate::util::result::{Error, Result};

use super::PermissionCalculator;

use num_enum::TryFromPrimitive;
use std::ops;

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Copy, Clone)]
#[repr(u32)]
pub enum ChannelPermission {
    View = 0b00000000000000000000000000000001,           // 1
    SendMessage = 0b00000000000000000000000000000010,    // 2
    ManageMessages = 0b00000000000000000000000000000100, // 4
    ManageChannel = 0b00000000000000000000000000001000,  // 8
    VoiceCall = 0b00000000000000000000000000010000,      // 16
    InviteOthers = 0b00000000000000000000000000100000,   // 32
    EmbedLinks = 0b00000000000000000000000001000000,     // 64
    UploadFiles = 0b00000000000000000000000010000000,   // 128
    Masquerade = 0b00000000000000000000000100000000,   // 256
}

lazy_static! {
    pub static ref DEFAULT_PERMISSION_DM: u32 =
        ChannelPermission::View
        + ChannelPermission::SendMessage
        + ChannelPermission::ManageChannel
        + ChannelPermission::VoiceCall
        + ChannelPermission::InviteOthers
        + ChannelPermission::EmbedLinks
        + ChannelPermission::UploadFiles
        + ChannelPermission::Masquerade;

    pub static ref DEFAULT_PERMISSION_SERVER: u32 =
        ChannelPermission::View
        + ChannelPermission::SendMessage
        + ChannelPermission::VoiceCall
        + ChannelPermission::InviteOthers
        + ChannelPermission::EmbedLinks
        + ChannelPermission::UploadFiles;
}

impl_op_ex!(+ |a: &ChannelPermission, b: &ChannelPermission| -> u32 { *a as u32 | *b as u32 });
impl_op_ex_commutative!(+ |a: &u32, b: &ChannelPermission| -> u32 { *a | *b as u32 });

bitfield! {
    pub struct ChannelPermissions(MSB0 [u32]);
    u32;
    pub get_view, _: 31;
    pub get_send_message, _: 30;
    pub get_manage_messages, _: 29;
    pub get_manage_channel, _: 28;
    pub get_voice_call, _: 27;
    pub get_invite_others, _: 26;
    pub get_embed_links, _: 25;
    pub get_upload_files, _: 24;
    pub get_masquerade, _: 23;
}

impl<'a> PermissionCalculator<'a> {
    pub async fn calculate_channel(self) -> Result<u32> {
        let channel = if let Some(channel) = self.channel {
            channel
        } else {
            unreachable!()
        };

        match channel {
            Channel::SavedMessages { user: owner, .. } => {
                if &self.perspective.id == owner {
                    Ok(u32::MAX)
                } else {
                    Ok(0)
                }
            }
            Channel::DirectMessage { recipients, .. } => {
                if recipients
                    .iter()
                    .find(|x| *x == &self.perspective.id)
                    .is_some()
                {
                    if let Some(recipient) = recipients.iter().find(|x| *x != &self.perspective.id)
                    {
                        let perms = self.for_user(recipient).await?;

                        if perms.get_send_message() {
                            return Ok(*DEFAULT_PERMISSION_DM);
                        }

                        return Ok(ChannelPermission::View as u32);
                    }
                }

                Ok(0)
            }
            Channel::Group { recipients, permissions, owner, .. } => {
                if &self.perspective.id == owner {
                    return Ok(*DEFAULT_PERMISSION_DM)
                }

                if recipients
                    .iter()
                    .find(|x| *x == &self.perspective.id)
                    .is_some()
                {
                    if let Some(permissions) = permissions {
                        Ok(permissions.clone() as u32)
                    } else {
                        Ok(*DEFAULT_PERMISSION_DM)
                    }
                } else {
                    Ok(0)
                }
            }
            Channel::TextChannel { server, default_permissions, role_permissions, .. }
            | Channel::VoiceChannel { server, default_permissions, role_permissions, .. } => {
                let server = Ref::from_unchecked(server.clone()).fetch_server().await?;

                if self.perspective.id == server.owner {
                    Ok(u32::MAX)
                } else {
                    match Ref::from_unchecked(self.perspective.id.clone()).fetch_member(&server.id).await {
                        Ok(member) => {
                            let mut perm = if let Some(permission) = default_permissions {
                                *permission as u32
                            } else {
                                server.default_permissions.1 as u32
                            };

                            if let Some(roles) = member.roles {
                                for role in roles {
                                    if let Some(permission) = role_permissions.get(&role) {
                                        perm |= *permission as u32;
                                    }

                                    if let Some(server_role) = server.roles.get(&role) {
                                        perm |= server_role.permissions.1 as u32;
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
        }
    }

    pub async fn for_channel(self) -> Result<ChannelPermissions<[u32; 1]>> {
        Ok(ChannelPermissions([self.calculate_channel().await?]))
    }
}
