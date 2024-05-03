mod permission;
mod user;

use bson::Bson;
pub use permission::*;
pub use user::*;

use serde::{Deserialize, Serialize};

/// Holds a permission value to manipulate.
#[derive(Debug)]
pub struct PermissionValue(u64);

/// Representation of a single permission override
#[derive(Deserialize, JsonSchema, Debug, Clone, Copy)]
pub struct Override {
    /// Allow bit flags
    allow: u64,
    /// Disallow bit flags
    deny: u64,
}

/// Representation of a single permission override
/// as it appears on models and in the database
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, Default)]
pub struct OverrideField {
    /// Allow bit flags
    a: i64,
    /// Disallow bit flags
    d: i64,
}

impl Override {
    /// Into allows
    pub fn allows(&self) -> u64 {
        self.allow
    }

    /// Into denies
    pub fn denies(&self) -> u64 {
        self.deny
    }
}

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

impl From<Override> for OverrideField {
    fn from(v: Override) -> Self {
        Self {
            a: v.allow as i64,
            d: v.deny as i64,
        }
    }
}

impl From<OverrideField> for Override {
    fn from(v: OverrideField) -> Self {
        Self {
            allow: v.a as u64,
            deny: v.d as u64,
        }
    }
}

impl From<OverrideField> for Bson {
    fn from(v: OverrideField) -> Self {
        Self::Document(bson::to_document(&v).unwrap())
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
