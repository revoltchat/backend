use revolt_result::Result;

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
    pub async fn delete(&self, db: &Database) -> Result<()> {
        // db.fetch_user(&self.id).await?.mark_deleted(db).await?;
        db.delete_bot(&self.id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::{Bot, FieldsBot};

    #[async_std::test]
    async fn crud() {
        let db = database_test!("bot_crud");

        let bot_id = "bot";
        let user_id = "user";
        let token = "my_token";

        let bot = Bot {
            id: bot_id.to_string(),
            owner: user_id.to_string(),
            token: token.to_string(),
            interactions_url: Some("some url".to_string()),
            ..Default::default()
        };

        db.insert_bot(&bot).await.unwrap();
        db.update_bot(
            bot_id,
            &crate::PartialBot {
                public: Some(true),
                ..Default::default()
            },
            vec![FieldsBot::Token, FieldsBot::InteractionsURL],
        )
        .await
        .unwrap();

        let fetched_bot1 = db.fetch_bot(bot_id).await.unwrap();
        let fetched_bot2 = db.fetch_bot_by_token(&fetched_bot1.token).await.unwrap();
        let fetched_bots = db.fetch_bots_by_user(user_id).await.unwrap();

        assert!(!bot.public);
        assert!(fetched_bot1.public);
        assert!(bot.interactions_url.is_some());
        assert!(fetched_bot1.interactions_url.is_none());
        assert_ne!(bot.token, fetched_bot1.token);
        assert_eq!(fetched_bot1, fetched_bot2);
        assert_eq!(fetched_bot1, fetched_bots[0]);
        assert_eq!(1, db.get_number_of_bots_by_user(user_id).await.unwrap());

        bot.delete(&db).await.unwrap();
        assert!(db.fetch_bot(bot_id).await.is_err());
        assert_eq!(0, db.get_number_of_bots_by_user(user_id).await.unwrap());
    }
}
