use crate::database::*;
use num_enum::TryFromPrimitive;
use std::ops;

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Copy, Clone)]
#[repr(u32)]
pub enum ChannelPermission {
    View = 1,
    SendMessage = 2,
}

bitfield! {
    pub struct ChannelPermissions(MSB0 [u32]);
    u32;
    pub get_view, _: 31;
    pub get_send_message, _: 30;
}

impl_op_ex!(+ |a: &ChannelPermission, b: &ChannelPermission| -> u32 { *a as u32 | *b as u32 });
impl_op_ex_commutative!(+ |a: &u32, b: &ChannelPermission| -> u32 { *a | *b as u32 });

pub async fn calculate(user: &User, target: &Channel) -> ChannelPermissions<[u32; 1]> {
    match target {
        Channel::SavedMessages { user: owner, .. } => {
            if &user.id == owner {
                ChannelPermissions([ ChannelPermission::View + ChannelPermission::SendMessage ])
            } else {
                ChannelPermissions([ 0 ])
            }
        }
        _ => unreachable!()
    }
}
