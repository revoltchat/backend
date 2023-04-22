auto_derived!(
    /// # Bot
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
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub flags: Option<i32>,
    }

    /// # Public Bot
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
);

#[cfg(feature = "from_database")]
impl PublicBot {
    pub fn from(
        bot: revolt_database::Bot,
        username: String,
        avatar: Option<String>,
        description: Option<String>,
    ) -> Self {
        PublicBot {
            id: bot.id,
            username,
            avatar: avatar.unwrap_or_default(),
            description: description.unwrap_or_default(),
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
            flags: value.flags,
        }
    }
}
