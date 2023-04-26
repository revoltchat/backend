use revolt_result::Result;

use crate::ReferenceDb;
use crate::{Bot, FieldsBot, PartialBot};

use super::AbstractBots;

#[async_trait]
impl AbstractBots for ReferenceDb {
    /// Insert new bot into the database
    async fn insert_bot(&self, bot: &Bot) -> Result<()> {
        let mut bots = self.bots.lock().await;
        if bots.contains_key(&bot.id) {
            Err(create_database_error!("insert", "bot"))
        } else {
            bots.insert(bot.id.to_string(), bot.clone());
            Ok(())
        }
    }

    /// Fetch a bot by its id
    async fn fetch_bot(&self, id: &str) -> Result<Bot> {
        let bots = self.bots.lock().await;
        bots.get(id).cloned().ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch a bot by its token
    async fn fetch_bot_by_token(&self, token: &str) -> Result<Bot> {
        let bots = self.bots.lock().await;
        bots.values()
            .find(|bot| bot.token == token)
            .cloned()
            .ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch bots owned by a user
    async fn fetch_bots_by_user(&self, user_id: &str) -> Result<Vec<Bot>> {
        let bots = self.bots.lock().await;
        Ok(bots
            .values()
            .filter(|bot| bot.owner == user_id)
            .cloned()
            .collect())
    }

    /// Get the number of bots owned by a user
    async fn get_number_of_bots_by_user(&self, user_id: &str) -> Result<usize> {
        let bots = self.bots.lock().await;
        Ok(bots.values().filter(|bot| bot.owner == user_id).count())
    }

    /// Update bot with new information
    async fn update_bot(
        &self,
        id: &str,
        partial: &PartialBot,
        remove: Vec<FieldsBot>,
    ) -> Result<()> {
        let mut bots = self.bots.lock().await;
        if let Some(bot) = bots.get_mut(id) {
            for field in remove {
                #[allow(clippy::disallowed_methods)]
                bot.remove_field(&field);
            }

            bot.apply_options(partial.clone());
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }

    /// Delete a bot from the database
    async fn delete_bot(&self, id: &str) -> Result<()> {
        let mut bots = self.bots.lock().await;
        if bots.remove(id).is_some() {
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }
}
