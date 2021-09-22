use crate::database::*;
use crate::util::result::{Error, Result};

use super::PermissionCalculator;

use mongodb::bson::doc;
use num_enum::TryFromPrimitive;
use std::ops;

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Copy, Clone)]
#[repr(u32)]
pub enum UserPermission {
    Access = 0b00000000000000000000000000000001,      // 1
    ViewProfile = 0b00000000000000000000000000000010, // 2
    SendMessage = 0b00000000000000000000000000000100, // 4
    Invite = 0b00000000000000000000000000001000,      // 8
}

bitfield! {
    pub struct UserPermissions(MSB0 [u32]);
    u32;
    pub get_access, _: 31;
    pub get_view_profile, _: 30;
    pub get_send_message, _: 29;
    pub get_invite, _: 28;
}

impl_op_ex!(+ |a: &UserPermission, b: &UserPermission| -> u32 { *a as u32 | *b as u32 });
impl_op_ex_commutative!(+ |a: &u32, b: &UserPermission| -> u32 { *a | *b as u32 });

pub fn get_relationship(a: &User, b: &str) -> RelationshipStatus {
    if a.id == b {
        return RelationshipStatus::User;
    }

    if let Some(relations) = &a.relations {
        if let Some(relationship) = relations.iter().find(|x| x.id == b) {
            return relationship.status.clone();
        }
    }

    RelationshipStatus::None
}

impl<'a> PermissionCalculator<'a> {
    pub async fn calculate_user(self, target: &str) -> Result<u32> {
        if &self.perspective.id == target {
            return Ok(u32::MAX);
        }

        let mut permissions: u32 = 0;
        match self
            .relationship
            .clone()
            .map(|v| v.to_owned())
            .unwrap_or_else(|| get_relationship(&self.perspective, &target))
        {
            RelationshipStatus::Friend | RelationshipStatus::User => return Ok(u32::MAX),
            RelationshipStatus::Blocked | RelationshipStatus::BlockedOther => {
                return Ok(UserPermission::Access as u32)
            }
            RelationshipStatus::Incoming | RelationshipStatus::Outgoing => {
                permissions = UserPermission::Access as u32;
                // ! INFO: if we add boolean switch for permission to
                // ! message people who have mutual, we need to get
                // ! rid of this return statement.
                // return Ok(permissions);
            }
            _ => {}
        }

        let check_server_overlap = async || {
            let server_ids = User::fetch_server_ids(&self.perspective.id).await?;

            Ok(
                db_conn()
                    .is_user_member_in_one_of_servers(&target, &server_ids)
                    .await
            )
        };

        if self.has_mutual_connection
            || check_server_overlap().await?
            || db_conn().are_users_connected_in_dms_or_group(&self.perspective.id, &target).await?
        {
            // ! FIXME: add privacy settings
            return Ok(UserPermission::Access + UserPermission::ViewProfile);
        }

        Ok(permissions)
    }

    pub async fn for_user(self, target: &str) -> Result<UserPermissions<[u32; 1]>> {
        Ok(UserPermissions([self.calculate_user(&target).await?]))
    }

    pub async fn for_user_given(self) -> Result<UserPermissions<[u32; 1]>> {
        let id = &self.user.unwrap().id;
        Ok(UserPermissions([self.calculate_user(&id).await?]))
    }
}
