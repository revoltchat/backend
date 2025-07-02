use iso8601_timestamp::Timestamp;

auto_derived!(
    /// Unique id of the user and bot
    pub struct AuthorizedBotId {
        /// User id
        pub user: String,

        /// Bot Id
        pub bot: String,
    }

    pub struct AuthorizedBot {
        /// Unique Id
        #[serde(rename = "_id")]
        pub id: AuthorizedBotId,

        /// When the authorized oauth2 bot connection was created at
        pub created_at: Timestamp,

        /// If and when the authorized oauth2 bot connection was revoked at
        pub deauthorized_at: Option<Timestamp>,

        /// Scopes the bot has access to
        pub scope: String
    }
);