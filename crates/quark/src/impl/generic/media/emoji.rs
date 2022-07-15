use std::{collections::HashSet, str::FromStr};

use ulid::Ulid;

use crate::{
    events::client::EventV1,
    models::{emoji::EmojiParent, Emoji},
    Database, Result,
};

lazy_static! {
    /// Permissible emojis
    static ref PERMISSIBLE_EMOJIS: HashSet<String> = include_str!(crate::asset!("emojis.txt"))
        .split('\n')
        .map(|x| x.into())
        .collect();
}

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

    /// Check whether we can use a given emoji
    pub async fn can_use(db: &Database, emoji: &str) -> Result<bool> {
        if Ulid::from_str(emoji).is_ok() {
            db.fetch_emoji(emoji).await?;
            Ok(true)
        } else {
            Ok(PERMISSIBLE_EMOJIS.contains(emoji))
        }
    }
}
