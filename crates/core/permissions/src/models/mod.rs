mod channel;
mod server;
mod user;

pub use channel::*;
pub use server::*;
pub use user::*;

/// Holds a permission value to manipulate.
#[derive(Debug)]
pub struct PermissionValue(u64);

impl PermissionValue {
    /// Apply a given override to this value
    pub fn apply(&mut self, v: Override) {
        self.allow(v.allow);
        self.revoke(v.deny);
    }

    /// Allow given permissions
    pub fn allow(&mut self, v: u64) {
        self.0 |= v;
    }

    /// Revoke given permissions
    pub fn revoke(&mut self, v: u64) {
        self.0 &= !v;
    }

    /// Restrict to given permissions
    pub fn restrict(&mut self, v: u64) {
        self.0 &= v;
    }
}

impl From<i64> for PermissionValue {
    fn from(v: i64) -> Self {
        Self(v as u64)
    }
}

impl From<u64> for PermissionValue {
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<PermissionValue> for u64 {
    fn from(v: PermissionValue) -> Self {
        v.0
    }
}

impl From<ChannelPermission> for PermissionValue {
    fn from(v: ChannelPermission) -> Self {
        (v as u64).into()
    }
}
