use revolt_result::Result;
use iso8601_timestamp::Timestamp;

use crate::{ReferenceDb, AuthorizedBot, AuthorizedBotId};

use super::AbstractAuthorizedBots;

#[async_trait]
impl AbstractAuthorizedBots for ReferenceDb {
    /// Insert an authorized bot into database.
    async fn insert_authorized_bot(&self, authorized_bot: &AuthorizedBot) -> Result<()> {
        let mut authorized_bots = self.authorized_bots.lock().await;

        if authorized_bots.contains_key(&authorized_bot.id) {
            Err(create_database_error!("insert", "authorized_bots"))
        } else {
            authorized_bots.insert(authorized_bot.id.clone(), authorized_bot.clone());
            Ok(())
        }
    }

    /// Fetch an authorized bot by its id
    async fn fetch_authorized_bot(&self, id: &AuthorizedBotId) -> Result<AuthorizedBot> {
        let authorized_bots = self.authorized_bots.lock().await;

        authorized_bots.get(id).cloned().ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch a users authorized bot by its id
    async fn fetch_users_authorized_bots(&self, user_id: &str) -> Result<Vec<AuthorizedBot>> {
        let authorized_bots = self.authorized_bots.lock().await;

        Ok(authorized_bots
            .values()
            .filter(|authorized_bot| authorized_bot.id.user == user_id)
            .cloned()
            .collect()
        )
    }

    /// Deletes an authorized bot
    async fn delete_authorized_bot(&self, id: &AuthorizedBotId) -> Result<()> {
        let mut authorized_bots = self.authorized_bots.lock().await;

        if authorized_bots.remove(id).is_some() {
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }

    /// Deauthorizes an authorized bot
    async fn deauthorize_authorized_bot(&self, id: &AuthorizedBotId) -> Result<AuthorizedBot> {
        let mut authorized_bots = self.authorized_bots.lock().await;

        if let Some(authorized_bot) = authorized_bots.get_mut(id) {
            authorized_bot.deauthorized_at = Some(Timestamp::now_utc());

            Ok(authorized_bot.clone())
        } else {
            Err(create_error!(NotFound))
        }
    }

    // Fetches all authorized bots which have been deauthorized
    async fn fetch_deauthorized_authorized_bots(&self) -> Result<Vec<AuthorizedBot>> {
        let authorized_bots = self.authorized_bots.lock().await;

        Ok(authorized_bots
            .values()
            .filter(|authorized_bot| authorized_bot.deauthorized_at.is_some())
            .cloned()
            .collect()
        )
    }
}