use super::{User, OAuth2Scope, OAuth2ScopeReasoning};
use std::collections::HashMap;

#[cfg(feature = "validator")]
use validator::Validate;

auto_derived!(
    /// Bot
    #[derive(Default)]
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

        /// Oauth2 bot settings
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "Option::is_none", default)
        )]
        pub oauth2: Option<BotOauth2>,

        /// Enum of bot flags
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "crate::if_zero_u32", default)
        )]
        pub flags: u32,
    }

    /// Optional fields on bot object
    pub enum FieldsBot {
        Token,
        InteractionsURL,
        Oauth2,
        Oauth2Secret,
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
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "String::is_empty", default)
        )]
        pub avatar: String,
        /// Profile Description
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "String::is_empty", default)
        )]
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
    #[derive(Default)]
    #[cfg_attr(feature = "validator", derive(Validate))]
    pub struct DataCreateBot {
        /// Bot username
        #[cfg_attr(
            feature = "validator",
            validate(length(min = 2, max = 32), regex = "super::RE_USERNAME")
        )]
        pub name: String,
    }

    /// New Bot Details
    #[derive(Default)]
    #[cfg_attr(feature = "validator", derive(Validate))]
    pub struct DataEditBot {
        /// Bot username
        #[cfg_attr(
            feature = "validator",
            validate(length(min = 2, max = 32), regex = "super::RE_USERNAME")
        )]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        /// Whether the bot can be added by anyone
        pub public: Option<bool>,
        /// Whether analytics should be gathered for this bot
        ///
        /// Must be enabled in order to show up on [Revolt Discover](https://rvlt.gg).
        pub analytics: Option<bool>,
        /// Interactions URL
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 2048)))]
        pub interactions_url: Option<String>,

        #[cfg_attr(feature = "validator", validate)]
        pub oauth2: Option<DataEditBotOauth2>,

        /// Fields to remove from bot object
        #[cfg_attr(feature = "serde", serde(default))]
        pub remove: Vec<FieldsBot>,
    }

    #[derive(Default)]
    #[cfg_attr(feature = "validator", derive(Validate))]
    pub struct DataEditBotOauth2 {
        pub public: Option<bool>,
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 10)))]
        pub redirects: Option<Vec<String>>,
        pub allowed_scopes: Option<HashMap<OAuth2Scope, OAuth2ScopeReasoning>>
    }

    /// Where we are inviting a bot to
    #[serde(untagged)]
    pub enum InviteBotDestination {
        /// Invite to a server
        Server {
            /// Server Id
            server: String,
        },
        /// Invite to a group
        Group {
            /// Group Id
            group: String,
        },
    }

    /// Owned Bots Response
    ///
    /// Both lists are sorted by their IDs.
    ///
    /// TODO: user should be in bot object
    pub struct OwnedBotsResponse {
        /// Bot objects
        pub bots: Vec<Bot>,
        /// User objects
        pub users: Vec<User>,
    }

    /// Bot with user response
    pub struct BotWithUserResponse {
        #[serde(flatten)]
        pub bot: Bot,
        pub user: User,
    }
);

auto_derived_partial!(
    /// Bot Oauth2 information
    pub struct BotOauth2 {
        /// Whether the oauth client is public and should not receive a secret key
        #[serde(default)]
        pub public: bool,
        /// Secret key used for authorisation, not provided if the client is public
        #[serde(default)]
        pub secret: Option<String>,
        /// Allowed redirects for the authorisation
        #[serde(default)]
        pub redirects: Vec<String>,
        /// Mapping of allowed scopes and the reasonings
        #[serde(default)]
        pub allowed_scopes: HashMap<OAuth2Scope, OAuth2ScopeReasoning>,
    },
    "PartialBotOauth2"
);