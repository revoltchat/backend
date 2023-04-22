use crate::Database;

auto_derived_partial!(
    /// Bot
    pub struct Bot {
        /// Bot Id
        ///
        /// This equals the associated bot user's id.
        #[serde(rename = "_id")]
        pub id: String,
        /// User Id of the bot owner
        pub owner: String,
        /// Token used to authenticate requests for this bot
        pub token: String,
        /// Whether the bot is public
        /// (may be invited by anyone)
        pub public: bool,

        /// Whether to enable analytics
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub analytics: bool,
        /// Whether this bot should be publicly discoverable
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub discoverable: bool,
        /// Reserved; URL for handling interactions
        #[serde(skip_serializing_if = "Option::is_none")]
        pub interactions_url: Option<String>,
        /// URL for terms of service
        #[serde(skip_serializing_if = "Option::is_none")]
        pub terms_of_service_url: Option<String>,
        /// URL for privacy policy
        #[serde(skip_serializing_if = "Option::is_none")]
        pub privacy_policy_url: Option<String>,

        /// Enum of bot flags
        #[serde(skip_serializing_if = "Option::is_none")]
        pub flags: Option<i32>,
    },
    "PartialBot"
);

auto_derived!(
    /// Flags that may be attributed to a bot
    #[repr(i32)]
    pub enum BotFlags {
        Verified = 1,
        Official = 2,
    }

    /// Optional fields on bot object
    pub enum FieldsBot {
        Token,
        InteractionsURL,
    }
);

impl Bot {
    /// Remove a field from this object
    pub fn remove(&mut self, field: &FieldsBot) {
        match field {
            FieldsBot::Token => self.token = nanoid::nanoid!(64),
            FieldsBot::InteractionsURL => {
                self.interactions_url.take();
            }
        }
    }

    /// Delete this bot
    pub async fn delete(&self, db: &Database) -> Result<(), ()> {
        // db.fetch_user(&self.id).await?.mark_deleted(db).await?;
        // db.delete_bot(&self.id).await
        Ok(())
    }
}
