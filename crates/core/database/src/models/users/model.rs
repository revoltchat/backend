use crate::File;

auto_derived_partial!(
    /// # User
    pub struct User {
        /// Unique Id
        #[serde(rename = "_id")]
        pub id: String,
        /// Username
        pub username: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        /// Avatar attachment
        pub avatar: Option<File>,
        /// Relationships with other users
        #[serde(skip_serializing_if = "Option::is_none")]
        pub relations: Option<Vec<Relationship>>,

        /// Bitfield of user badges
        #[serde(skip_serializing_if = "Option::is_none")]
        pub badges: Option<i32>,
        /// User's current status
        #[serde(skip_serializing_if = "Option::is_none")]
        pub status: Option<UserStatus>,
        /// User's profile page
        #[serde(skip_serializing_if = "Option::is_none")]
        pub profile: Option<UserProfile>,

        /// Enum of user flags
        #[serde(skip_serializing_if = "Option::is_none")]
        pub flags: Option<i32>,
        /// Whether this user is privileged
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub privileged: bool,
        /// Bot information
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bot: Option<BotInformation>,
    },
    "PartialUser"
);

auto_derived!(
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

    /// Relationship entry indicating current status with other user
    pub struct Relationship {
        #[serde(rename = "_id")]
        pub id: String,
        pub status: RelationshipStatus,
    }

    /// Presence status
    pub enum Presence {
        /// User is online
        Online,
        /// User is not currently available
        Idle,
        /// User is focusing / will only receive mentions
        Focus,
        /// User is busy / will not receive any notifications
        Busy,
        /// User appears to be offline
        Invisible,
    }

    /// User's active status
    pub struct UserStatus {
        /// Custom status text
        #[serde(skip_serializing_if = "String::is_empty")]
        pub text: String,
        /// Current presence option
        #[serde(skip_serializing_if = "Option::is_none")]
        pub presence: Option<Presence>,
    }

    /// User's profile
    pub struct UserProfile {
        /// Text content on user's profile
        #[serde(skip_serializing_if = "String::is_empty")]
        pub content: String,
        /// Background visible on user's profile
        #[serde(skip_serializing_if = "Option::is_none")]
        pub background: Option<File>,
    }

    /// Bot information for if the user is a bot
    pub struct BotInformation {
        /// Id of the owner of this bot
        pub owner: String,
    }
);
