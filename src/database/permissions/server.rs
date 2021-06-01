use crate::database::*;
use crate::util::result::Result;

use super::PermissionCalculator;

use num_enum::TryFromPrimitive;
use std::ops;

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Copy, Clone)]
#[repr(u32)]
pub enum ServerPermission {
    View = 1,
}

impl_op_ex!(+ |a: &ServerPermission, b: &ServerPermission| -> u32 { *a as u32 | *b as u32 });
impl_op_ex_commutative!(+ |a: &u32, b: &ServerPermission| -> u32 { *a | *b as u32 });

bitfield! {
    pub struct ServerPermissions(MSB0 [u32]);
    u32;
    pub get_view, _: 31;
}

impl<'a> PermissionCalculator<'a> {
    pub async fn calculate_server(self) -> Result<u32> {
        let server = if let Some(server) = self.server {
            server
        } else {
            unreachable!()
        };

        if &self.perspective.id == server.owner {
            Ok(u32::MAX)
        } else {
            Ok(ServerPermission::View as u32)
        }
    }

    pub async fn for_server(self) -> Result<ChannelPermissions<[u32; 1]>> {
        Ok(ServerPermissions([self.calculate_server().await?]))
    }
}
