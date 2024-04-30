use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use std::ops;

/// User permission definitions
#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, TryFromPrimitive, Copy, Clone, JsonSchema,
)]
#[repr(u32)]
pub enum UserPermission {
    Access = 1 << 0,
    ViewProfile = 1 << 1,
    SendMessage = 1 << 2,
    Invite = 1 << 3,
}

impl_op_ex!(+ |a: &UserPermission, b: &UserPermission| -> u32 { *a as u32 | *b as u32 });
impl_op_ex_commutative!(+ |a: &u32, b: &UserPermission| -> u32 { *a | *b as u32 });

bitfield! {
    pub struct UserPermissions(MSB0 [u32]);
    u32;
    pub get_access, _: 31;
    pub get_view_profile, _: 30;
    pub get_send_message, _: 29;
    pub get_invite, _: 28;
}

pub type UserPerms = UserPermissions<[u32; 1]>;
