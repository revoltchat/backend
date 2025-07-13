use iso8601_timestamp::Timestamp;

use crate::v0::{PublicBot, OAuth2Scope};

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
        pub scope: Vec<OAuth2Scope>,
    }

    pub struct AuthorizedBotsResponse {
        pub public_bot: PublicBot,
        pub authorized_bot: AuthorizedBot,
    }
);