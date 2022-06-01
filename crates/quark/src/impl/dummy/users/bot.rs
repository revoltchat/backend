use crate::models::bot::{Bot, FieldsBot, PartialBot};
use crate::{AbstractBot, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractBot for DummyDb {
    async fn fetch_bot(&self, id: &str) -> Result<Bot> {
        Ok(Bot {
            id: id.into(),
            owner: "user".into(),
            token: "token".into(),
            public: true,
            analytics: true,
            discoverable: true,
            ..Default::default()
        })
    }

    async fn fetch_bot_by_token(&self, _token: &str) -> Result<Bot> {
        self.fetch_bot("bot").await
    }

    async fn insert_bot(&self, bot: &Bot) -> Result<()> {
        info!("Insert {bot:?}");
        Ok(())
    }

    async fn update_bot(&self, id: &str, bot: &PartialBot, remove: Vec<FieldsBot>) -> Result<()> {
        info!("Update {id} with {bot:?} and remove {remove:?}");
        Ok(())
    }

    async fn delete_bot(&self, id: &str) -> Result<()> {
        info!("Delete {id}");
        Ok(())
    }

    async fn fetch_bots_by_user(&self, user_id: &str) -> Result<Vec<Bot>> {
        Ok(vec![self.fetch_bot(user_id).await.unwrap()])
    }

    async fn get_number_of_bots_by_user(&self, _user_id: &str) -> Result<usize> {
        Ok(1)
    }
}
