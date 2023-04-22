use ::mongodb::bson::doc;
use revolt_result::Result;

use crate::{Bot, FieldsBot, PartialBot};
use crate::{IntoDocumentPath, MongoDb};

use super::AbstractBots;

static COL: &str = "bots";

#[async_trait]
impl AbstractBots for MongoDb {
    /// Fetch a bot by its id
    async fn fetch_bot(&self, id: &str) -> Result<Bot> {
        self.find_one_by_id(COL, id)
            .await
            .map_err(|_| create_database_error!("find_one", COL))?
            .ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch a bot by its token
    async fn fetch_bot_by_token(&self, token: &str) -> Result<Bot> {
        self.find_one(
            COL,
            doc! {
                "token": token
            },
        )
        .await
        .map_err(|_| create_database_error!("find_one", COL))?
        .ok_or_else(|| create_error!(NotFound))
    }

    /// Insert new bot into the database
    async fn insert_bot(&self, bot: &Bot) -> Result<()> {
        self.insert_one(COL, &bot)
            .await
            .map(|_| ())
            .map_err(|_| create_database_error!("insert_one", COL))
    }

    /// Update bot with new information
    async fn update_bot(
        &self,
        id: &str,
        partial: &PartialBot,
        remove: Vec<FieldsBot>,
    ) -> Result<()> {
        self.update_one_by_id(
            COL,
            id,
            partial,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            None,
        )
        .await
        .map(|_| ())
        .map_err(|_| create_database_error!("update_one", COL))
    }

    /// Delete a bot from the database
    async fn delete_bot(&self, id: &str) -> Result<()> {
        self.delete_one_by_id(COL, id)
            .await
            .map(|_| ())
            .map_err(|_| create_database_error!("delete_one", COL))
    }

    /// Fetch bots owned by a user
    async fn fetch_bots_by_user(&self, user_id: &str) -> Result<Vec<Bot>> {
        self.find(
            COL,
            doc! {
                "owner": user_id
            },
        )
        .await
        .map_err(|_| create_database_error!("find", COL))
    }

    /// Get the number of bots owned by a user
    async fn get_number_of_bots_by_user(&self, user_id: &str) -> Result<usize> {
        self.count_documents(
            COL,
            doc! {
                "owner": user_id
            },
        )
        .await
        .map(|v| v as usize)
        .map_err(|_| create_database_error!("count", COL))
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
