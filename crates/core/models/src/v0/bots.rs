use super::User;

auto_derived!(
    /// Bot
    pub struct Bot {
        /// Bot Id
        #[cfg_attr(feature = "serde", serde(rename = "_id"))]
        pub id: String,

        /// User Id of the bot owner
        #[cfg_attr(feature = "serde", serde(rename = "owner"))]
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
        #[cfg_attr(feature = "serde", serde(rename = "_id"))]
        pub id: String,

        /// Bot Username
        pub username: String,
        /// Profile Avatar
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "String::is_empty"))]
        pub avatar: String,
        /// Profile Description
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "String::is_empty"))]
        pub description: String,
    }

    /// Bot Response
    pub struct FetchBotResponse {
        /// Bot object
        pub bot: Bot,
        /// User object
        pub user: User,
    }

    /// Bot Details
    pub struct DataCreateBot {
        /// Bot username
        #[cfg_attr(
            feature = "validator",
            validate(length(min = 2, max = 32), regex = "RE_USERNAME")
        )]
        name: String,
    }
);
