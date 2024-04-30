use crate::models::bot::{Bot, FieldsBot, PartialBot};
use crate::r#impl::mongo::IntoDocumentPath;
use crate::{AbstractBot, Result};

use super::super::MongoDb;

static COL: &str = "bots";

#[async_trait]
impl AbstractBot for MongoDb {
    async fn fetch_bot(&self, id: &str) -> Result<Bot> {
        self.find_one_by_id(COL, id).await
    }

    async fn fetch_bot_by_token(&self, token: &str) -> Result<Bot> {
        self.find_one(
            COL,
            doc! {
                "token": token
            },
        )
        .await
    }

    async fn insert_bot(&self, bot: &Bot) -> Result<()> {
        self.insert_one(COL, &bot).await.map(|_| ())
    }

    async fn update_bot(&self, id: &str, bot: &PartialBot, remove: Vec<FieldsBot>) -> Result<()> {
        self.update_one_by_id(
            COL,
            id,
            bot,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            None,
        )
        .await
        .map(|_| ())
    }

    async fn delete_bot(&self, id: &str) -> Result<()> {
        self.delete_one_by_id(COL, id).await.map(|_| ())
    }

    async fn fetch_bots_by_user(&self, user_id: &str) -> Result<Vec<Bot>> {
        self.find(
            COL,
            doc! {
                "owner": user_id
            },
        )
        .await
    }

    async fn get_number_of_bots_by_user(&self, user_id: &str) -> Result<usize> {
        // ! FIXME: move this to generic?
        self.fetch_bots_by_user(user_id).await.map(|x| x.len())
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
