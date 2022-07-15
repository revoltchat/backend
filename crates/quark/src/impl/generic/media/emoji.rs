use crate::{
    events::client::EventV1,
    models::{emoji::EmojiParent, Emoji},
    Database, Result,
};

impl Emoji {
    /// Get parent id
    fn parent(&self) -> &str {
        match &self.parent {
            EmojiParent::Server { id } => id,
            EmojiParent::Detached => "",
        }
    }

    /// Create an emoji
    pub async fn create(&self, db: &Database) -> Result<()> {
        db.insert_emoji(self).await?;
        EventV1::EmojiCreate(self.clone())
            .p(self.parent().to_string())
            .await;

        Ok(())
    }

    /// Delete an emoji
    pub async fn delete(self, db: &Database) -> Result<()> {
        EventV1::EmojiDelete {
            id: self.id.to_string(),
        }
        .p(self.parent().to_string())
        .await;

        db.detach_emoji(&self).await
    }
}
