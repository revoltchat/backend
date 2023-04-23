use crate::File;

auto_derived!(
    /// User
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
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub relations: Vec<Relationship>,

        /// Bitfield of user badges
        #[serde(skip_serializing_if = "crate::if_zero_u32", default)]
        pub badges: u32,
        /// User's current status
        #[serde(skip_serializing_if = "Option::is_none")]
        pub status: Option<UserStatus>,
        /// User's profile page
        #[serde(skip_serializing_if = "Option::is_none")]
        pub profile: Option<UserProfile>,

        /// Enum of user flags
        #[serde(skip_serializing_if = "crate::if_zero_u32", default)]
        pub flags: u32,
        /// Whether this user is privileged
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub privileged: bool,
        /// Bot information
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bot: Option<BotInformation>,

        /// Current session user's relationship with this user
        pub relationship: RelationshipStatus,
        /// Whether this user is currently online
        pub online: bool,
    }

    /// User's relationship with another user (or themselves)
    #[derive(Default)]
    pub enum RelationshipStatus {
        #[default]
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
        pub user_id: String,
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

    /// User badge bitfield
    #[repr(u32)]
    pub enum UserBadges {
        /// Revolt Developer
        Developer = 1,
        /// Helped translate Revolt
        Translator = 2,
        /// Monetarily supported Revolt
        Supporter = 4,
        /// Responsibly disclosed a security issue
        ResponsibleDisclosure = 8,
        /// Revolt Founder
        Founder = 16,
        /// Platform moderator
        PlatformModeration = 32,
        /// Active monetary supporter
        ActiveSupporter = 64,
        /// 🦊🦝
        Paw = 128,
        /// Joined as one of the first 1000 users in 2021
        EarlyAdopter = 256,
        /// Amogus
        ReservedRelevantJokeBadge1 = 512,
        /// Low resolution troll face
        ReservedRelevantJokeBadge2 = 1024,
    }

    /// User flag enum
    #[repr(u32)]
    pub enum UserFlags {
        /// User has been suspended from the platform
        Suspended = 1,
        /// User has deleted their account
        Deleted = 2,
        /// User was banned off the platform
        Banned = 4,
        /// User was marked as spam and removed from platform
        Spam = 8,
    }

    /// Bot information for if the user is a bot
    pub struct BotInformation {
        /// Id of the owner of this bot
        #[serde(rename = "owner")]
        pub owner_id: String,
    }
);

pub trait CheckRelationship {
    fn with(&self, user: &str) -> RelationshipStatus;
}

impl CheckRelationship for Vec<Relationship> {
    fn with(&self, user: &str) -> RelationshipStatus {
        for entry in self {
            if entry.user_id == user {
                return entry.status.clone();
            }
        }

        RelationshipStatus::None
    }
}

#[cfg(feature = "from_database")]
impl User {
    pub async fn from<P>(user: revolt_database::User, perspective: P) -> Self
    where
        P: Into<Option<revolt_database::User>>,
    {
        let relationship = if let Some(perspective) = perspective.into() {
            perspective
                .relations
                .unwrap_or_default()
                .into_iter()
                .find(|relationship| relationship.id == user.id)
                .map(|relationship| relationship.status.into())
                .unwrap_or_default()
        } else {
            RelationshipStatus::None
        };

        // do permission stuff here
        // TODO: implement permissions =)
        let can_see_profile = false;

        Self {
            username: user.username,
            avatar: user.avatar.map(|file| file.into()),
            relations: vec![],
            badges: user.badges.unwrap_or_default() as u32,
            status: None,
            profile: None,
            flags: user.flags.unwrap_or_default() as u32,
            privileged: user.privileged,
            bot: user.bot.map(|bot| bot.into()),
            relationship,
            online: can_see_profile && revolt_presence::is_online(&user.id).await,
            id: user.id,
        }
    }
}

#[cfg(feature = "from_database")]
impl From<revolt_database::BotInformation> for BotInformation {
    fn from(value: revolt_database::BotInformation) -> Self {
        BotInformation {
            owner_id: value.owner,
        }
    }
}

#[cfg(feature = "from_database")]
impl From<revolt_database::Relationship> for Relationship {
    fn from(value: revolt_database::Relationship) -> Self {
        Self {
            user_id: value.id,
            status: value.status.into(),
        }
    }
}

#[cfg(feature = "from_database")]
impl From<revolt_database::RelationshipStatus> for RelationshipStatus {
    fn from(value: revolt_database::RelationshipStatus) -> Self {
        match value {
            revolt_database::RelationshipStatus::None => RelationshipStatus::None,
            revolt_database::RelationshipStatus::User => RelationshipStatus::User,
            revolt_database::RelationshipStatus::Friend => RelationshipStatus::Friend,
            revolt_database::RelationshipStatus::Outgoing => RelationshipStatus::Outgoing,
            revolt_database::RelationshipStatus::Incoming => RelationshipStatus::Incoming,
            revolt_database::RelationshipStatus::Blocked => RelationshipStatus::Blocked,
            revolt_database::RelationshipStatus::BlockedOther => RelationshipStatus::BlockedOther,
        }
    }
}
