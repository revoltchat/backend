use super::File;

use iso8601_timestamp::Timestamp;
use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for valid role colours
///
/// Allows the use of named colours, rgb(a), variables and all gradients.
///
/// Flags:
/// - Case-insensitive (`i`)
///
/// Source:
/// ```regex
/// VALUE = [a-z ]+|var\(--[a-z\d-]+\)|rgba?\([\d, ]+\)|#[a-f0-9]+
/// ADDITIONAL_VALUE = \d+deg
/// STOP = ([ ]+(\d{1,3}%|0))?
///
/// ^(?:VALUE|(repeating-)?(linear|conic|radial)-gradient\((VALUE|ADDITIONAL_VALUE)STOP(,[ ]*(VALUE)STOP)+\))$
/// ```
pub static RE_COLOUR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?:[a-z ]+|var\(--[a-z\d-]+\)|rgba?\([\d, ]+\)|#[a-f0-9]+|(repeating-)?(linear|conic|radial)-gradient\(([a-z ]+|var\(--[a-z\d-]+\)|rgba?\([\d, ]+\)|#[a-f0-9]+|\d+deg)([ ]+(\d{1,3}%|0))?(,[ ]*([a-z ]+|var\(--[a-z\d-]+\)|rgba?\([\d, ]+\)|#[a-f0-9]+)([ ]+(\d{1,3}%|0))?)+\))$").unwrap()
});

auto_derived_partial!(
    /// Server Member
    pub struct Member {
        /// Unique member id
        #[cfg_attr(feature = "serde", serde(rename = "_id"))]
        pub id: MemberCompositeKey,

        /// Time at which this user joined the server
        pub joined_at: Timestamp,

        /// Member's nickname
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub nickname: Option<String>,
        /// Avatar attachment
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub avatar: Option<File>,

        /// Member's roles
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "Vec::is_empty", default)
        )]
        pub roles: Vec<String>,
        /// Timestamp this member is timed out until
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub timeout: Option<Timestamp>,
    },
    "PartialMember"
);

auto_derived!(
    /// Composite primary key consisting of server and user id
    #[derive(Hash, Default)]
    pub struct MemberCompositeKey {
        /// Server Id
        pub server: String,
        /// User Id
        pub user: String,
    }

    /// Optional fields on server member object
    pub enum FieldsMember {
        Nickname,
        Avatar,
        Roles,
        Timeout,
    }

    /// Member removal intention
    pub enum RemovalIntention {
        Leave,
        Kick,
        Ban,
    }
);
