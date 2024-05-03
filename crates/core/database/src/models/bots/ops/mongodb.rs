use revolt_result::Result;

use crate::{Bot, FieldsBot, PartialBot};
use crate::{IntoDocumentPath, MongoDb};

use super::AbstractBots;

static COL: &str = "bots";

#[async_trait]
impl AbstractBots for MongoDb {
    /// Insert new bot into the database
    async fn insert_bot(&self, bot: &Bot) -> Result<()> {
        query!(self, insert_one, COL, &bot).map(|_| ())
    }

    /// Fetch a bot by its id
    async fn fetch_bot(&self, id: &str) -> Result<Bot> {
        query!(self, find_one_by_id, COL, id)?.ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch a bot by its token
    async fn fetch_bot_by_token(&self, token: &str) -> Result<Bot> {
        query!(
            self,
            find_one,
            COL,
            doc! {
                "token": token
            }
        )?
        .ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch bots owned by a user
    async fn fetch_bots_by_user(&self, user_id: &str) -> Result<Vec<Bot>> {
        query!(
            self,
            find,
            COL,
            doc! {
                "owner": user_id
            }
        )
    }

    /// Get the number of bots owned by a user
    async fn get_number_of_bots_by_user(&self, user_id: &str) -> Result<usize> {
        query!(
            self,
            count_documents,
            COL,
            doc! {
                "owner": user_id
            }
        )
        .map(|v| v as usize)
    }

    /// Update bot with new information
    async fn update_bot(
        &self,
        id: &str,
        partial: &PartialBot,
        remove: Vec<FieldsBot>,
    ) -> Result<()> {
        query!(
            self,
            update_one_by_id,
            COL,
            id,
            partial,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            None
        )
        .map(|_| ())
    }

    /// Delete a bot from the database
    async fn delete_bot(&self, id: &str) -> Result<()> {
        query!(self, delete_one_by_id, COL, id).map(|_| ())
    }
}

impl IntoDocumentPath for FieldsBot {
    fn as_path(&self) -> Option<&'static str> {
        match self {
            FieldsBot::InteractionsURL => Some("interactions_url"),
            FieldsBot::Token => None,
        }
    }
}
