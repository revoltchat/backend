use crate::models::message::{AppendMessage, Message, MessageSort, PartialMessage};
use crate::{AbstractMessage, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractMessage for DummyDb {
    async fn fetch_message(&self, id: &str) -> Result<Message> {
        Ok(Message {
            id: id.into(),
            channel: "channel".into(),
            author: "author".into(),
            content: Some("message content".into()),

            ..Default::default()
        })
    }

    async fn insert_message(&self, message: &Message) -> Result<()> {
        info!("Insert {message:?}");
        Ok(())
    }

    async fn update_message(&self, id: &str, message: &PartialMessage) -> Result<()> {
        info!("Update {id} with {message:?}");
        Ok(())
    }

    async fn append_message(&self, id: &str, append: &AppendMessage) -> Result<()> {
        info!("Append {id} with {append:?}");
        Ok(())
    }

    async fn delete_message(&self, id: &str) -> Result<()> {
        info!("Delete {id}");
        Ok(())
    }

    async fn delete_messages(&self, channel: &str, ids: Vec<String>) -> Result<()> {
        info!("Delete {ids:?} in {channel}");
        Ok(())
    }

    async fn fetch_messages(
        &self,
        channel: &str,
        _limit: Option<i64>,
        _before: Option<String>,
        _after: Option<String>,
        _sort: Option<MessageSort>,
        _nearby: Option<String>,
    ) -> Result<Vec<Message>> {
        Ok(vec![self.fetch_message(channel).await.unwrap()])
    }

    async fn search_messages(
        &self,
        channel: &str,
        _query: &str,
        _limit: Option<i64>,
        _before: Option<String>,
        _after: Option<String>,
        _sort: MessageSort,
    ) -> Result<Vec<Message>> {
        Ok(vec![self.fetch_message(channel).await.unwrap()])
    }

    /// Add a new reaction to a message
    async fn add_reaction(&self, id: &str, emoji: &str, user: &str) -> Result<()> {
        info!("Add to {id} with {emoji} and {user}");
        Ok(())
    }

    /// Remove a reaction from a message
    async fn remove_reaction(&self, id: &str, emoji: &str, user: &str) -> Result<()> {
        info!("Remove {emoji} from {id} for {user}");
        Ok(())
    }

    /// Remove reaction from a message
    async fn clear_reaction(&self, id: &str, emoji: &str) -> Result<()> {
        info!("Clear {emoji} on {id}");
        Ok(())
    }
}
