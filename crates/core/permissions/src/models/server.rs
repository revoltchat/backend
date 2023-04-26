/// Representation of a single permission override
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Override {
    /// Allow bit flags
    pub allow: u64,
    /// Disallow bit flags
    pub deny: u64,
}

/// Representation of a single permission override
/// as it appears on models and in the database
#[derive(/*JsonSchema, */ Debug, Clone, Copy, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

/*impl From<OverrideField> for Bson {
    fn from(v: OverrideField) -> Self {
        Self::Document(bson::to_document(&v).unwrap())
    }
}*/
