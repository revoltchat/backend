use crate::models::Emoji;
use crate::Result;

#[async_trait]
pub trait AbstractEmoji: Sync + Send {
    /// Fetch an emoji by its id
    async fn fetch_emoji(&self, id: &str) -> Result<Emoji>;

    /// Fetch emoji by their parent id
    async fn fetch_emoji_by_parent_id(&self, parent_id: &str) -> Result<Vec<Emoji>>;

    /// Fetch emoji by their parent ids
    async fn fetch_emoji_by_parent_ids(&self, parent_ids: &[String]) -> Result<Vec<Emoji>>;

    /// Insert emoji into database.
    async fn insert_emoji(&self, emoji: &Emoji) -> Result<()>;

    /// Detach an emoji by its id
    async fn detach_emoji(&self, emoji: &Emoji) -> Result<()>;
}
