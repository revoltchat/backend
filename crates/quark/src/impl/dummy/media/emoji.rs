use crate::models::emoji::EmojiParent;
use crate::models::Emoji;
use crate::{AbstractEmoji, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractEmoji for DummyDb {
    /// Fetch an emoji by its id
    async fn fetch_emoji(&self, id: &str) -> Result<Emoji> {
        Ok(Emoji {
            id: id.into(),
            name: id.into(),
            parent: EmojiParent::Server { id: id.into() },
            creator_id: id.into(),
            animated: false,
            nsfw: false,
        })
    }

    /// Fetch emoji by their ids
    async fn fetch_emoji_by_parent_id(&self, parent_id: &str) -> Result<Vec<Emoji>> {
        Ok(vec![self.fetch_emoji(parent_id).await?])
    }

    /// Fetch emoji by their parent ids
    async fn fetch_emoji_by_parent_ids(&self, _parent_ids: &[String]) -> Result<Vec<Emoji>> {
        Ok(vec![])
    }

    /// Insert emoji into database.
    async fn insert_emoji(&self, emoji: &Emoji) -> Result<()> {
        info!("Insert {emoji:?}");
        Ok(())
    }

    /// Detach an emoji by its id
    async fn detach_emoji(&self, emoji: &Emoji) -> Result<()> {
        info!("Detach {emoji:?}");
        Ok(())
    }
}
