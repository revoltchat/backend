use crate::models::message::{AppendMessage, Message, MessageSort, PartialMessage};
use crate::Result;

#[async_trait]
pub trait AbstractMessage: Sync + Send {
    /// Fetch a message by its id
    async fn fetch_message(&self, id: &str) -> Result<Message>;

    /// Insert a new message into the database
    async fn insert_message(&self, message: &Message) -> Result<()>;

    /// Update a given message with new information
    async fn update_message(&self, id: &str, message: &PartialMessage) -> Result<()>;

    /// Append information to a given message
    async fn append_message(&self, id: &str, append: &AppendMessage) -> Result<()>;

    /// Delete a message from the database by its id
    async fn delete_message(&self, id: &str) -> Result<()>;

    /// Delete messages from a channel by their ids and corresponding channel id
    async fn delete_messages(&self, channel: &str, ids: Vec<String>) -> Result<()>;

    /// Fetch multiple messages
    async fn fetch_messages(
        &self,
        channel: &str,
        limit: Option<i64>,
        before: Option<String>,
        after: Option<String>,
        sort: Option<MessageSort>,
        nearby: Option<String>,
    ) -> Result<Vec<Message>>;

    /// Search for messages
    async fn search_messages(
        &self,
        channel: &str,
        query: &str,
        limit: Option<i64>,
        before: Option<String>,
        after: Option<String>,
        sort: MessageSort,
    ) -> Result<Vec<Message>>;

    /// Add a new reaction to a message
    async fn add_reaction(&self, id: &str, emoji: &str, user: &str) -> Result<()>;

    /// Remove a reaction from a message
    async fn remove_reaction(&self, id: &str, emoji: &str, user: &str) -> Result<()>;

    /// Remove reaction from a message
    async fn clear_reaction(&self, id: &str, emoji: &str) -> Result<()>;
}
