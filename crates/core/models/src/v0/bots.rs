use super::User;

auto_derived!(
    /// Bot
    pub struct Bot {
        /// Bot Id
        #[serde(rename = "_id")]
        pub id: String,

        /// User Id of the bot owner
        #[serde(rename = "owner")]
        pub owner_id: String,
        /// Token used to authenticate requests for this bot
        pub token: String,
        /// Whether the bot is public
        /// (may be invited by anyone)
        pub public: bool,

        /// Whether to enable analytics
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "crate::if_false", default)
        )]
        pub analytics: bool,
        /// Whether this bot should be publicly discoverable
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "crate::if_false", default)
        )]
        pub discoverable: bool,
        /// Reserved; URL for handling interactions
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "String::is_empty", default)
        )]
        pub interactions_url: String,
        /// URL for terms of service
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "String::is_empty", default)
        )]
        pub terms_of_service_url: String,
        /// URL for privacy policy
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "String::is_empty", default)
        )]
        pub privacy_policy_url: String,

        /// Enum of bot flags
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "crate::if_zero_u32", default)
        )]
        pub flags: u32,
    }

    /// Flags that may be attributed to a bot
    #[repr(u32)]
    pub enum BotFlags {
        Verified = 1,
        Official = 2,
    }

    /// Public Bot
    pub struct PublicBot {
        /// Bot Id
        #[serde(rename = "_id")]
        id: String,

        /// Bot Username
        username: String,
        /// Profile Avatar
        #[serde(skip_serializing_if = "String::is_empty")]
        avatar: String,
        /// Profile Description
        #[serde(skip_serializing_if = "String::is_empty")]
        description: String,
    }

    /// Bot Response
    pub struct FetchBotResponse {
        /// Bot object
        pub bot: Bot,
        /// User object
        pub user: User,
    }
);

#[cfg(feature = "from_database")]
impl PublicBot {
    pub fn from(bot: revolt_database::Bot, user: revolt_database::User) -> Self {
        #[cfg(debug_assertions)]
        assert_eq!(bot.id, user.id);

        PublicBot {
            id: bot.id,
            username: user.username,
            avatar: user.avatar.map(|x| x.id).unwrap_or_default(),
            description: user
                .profile
                .map(|profile| profile.content)
                .unwrap_or_default(),
        }
    }
}

#[cfg(feature = "from_database")]
impl From<revolt_database::Bot> for Bot {
    fn from(value: revolt_database::Bot) -> Self {
        Bot {
            id: value.id,
            owner_id: value.owner,
            token: value.token,
            public: value.public,
            analytics: value.analytics,
            discoverable: value.discoverable,
            interactions_url: value.interactions_url,
            terms_of_service_url: value.terms_of_service_url,
            privacy_policy_url: value.privacy_policy_url,
            flags: value.flags.unwrap_or_default() as u32,
        }
    }
}
