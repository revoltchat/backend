use std::fmt;

/// User's relationship with another user (or themselves)
pub enum RelationshipStatus {
    None,
    User,
    Friend,
    Outgoing,
    Incoming,
    Blocked,
    BlockedOther,
}

/// User permission definitions
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "try-from-primitive", derive(num_enum::TryFromPrimitive))]
#[repr(u32)]
pub enum UserPermission {
    Access = 1 << 0,
    ViewProfile = 1 << 1,
    SendMessage = 1 << 2,
    Invite = 1 << 3,
}

impl fmt::Display for UserPermission {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl_op_ex!(+ |a: &UserPermission, b: &UserPermission| -> u32 { *a as u32 | *b as u32 });
impl_op_ex_commutative!(+ |a: &u32, b: &UserPermission| -> u32 { *a | *b as u32 });
