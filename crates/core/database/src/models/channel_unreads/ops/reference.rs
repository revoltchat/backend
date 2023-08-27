use revolt_result::Result;
use ulid::Ulid;

use crate::{ChannelCompositeKey, ChannelUnread, ReferenceDb};

use super::AbstractChannelUnreads;

#[async_trait]
impl AbstractChannelUnreads for ReferenceDb {
    /// Acknowledge a message.
    async fn acknowledge_message(
        &self,
        channel_id: &str,
        user_id: &str,
        message_id: &str,
    ) -> Result<()> {
        let mut unreads = self.channel_unreads.lock().await;
        let key = ChannelCompositeKey {
            channel: channel_id.to_string(),
            user: user_id.to_string(),
        };

        if let Some(unread) = unreads.get_mut(&key) {
            unread.mentions = None;
            unread.last_id.replace(message_id.to_string());
        } else {
            unreads.insert(
                key.clone(),
                ChannelUnread {
                    id: key,
                    last_id: Some(message_id.to_string()),
                    mentions: None,
                },
            );
        }

        Ok(())
    }

    /// Acknowledge many channels.
    async fn acknowledge_channels(&self, user_id: &str, channel_ids: &[String]) -> Result<()> {
        let current_time = Ulid::new().to_string();
        for channel_id in channel_ids {
            #[allow(clippy::disallowed_methods)]
            self.acknowledge_message(channel_id, user_id, &current_time)
                .await?;
        }

        Ok(())
    }

    /// Add a mention.
    async fn add_mention_to_unread<'a>(
        &self,
        channel_id: &str,
        user_id: &str,
        message_ids: &[String],
    ) -> Result<()> {
        let mut unreads = self.channel_unreads.lock().await;
        let key = ChannelCompositeKey {
            channel: channel_id.to_string(),
            user: user_id.to_string(),
        };

        if let Some(unread) = unreads.get_mut(&key) {
            unread.mentions.replace(message_ids.to_vec());
        } else {
            unreads.insert(
                key.clone(),
                ChannelUnread {
                    id: key,
                    last_id: None,
                    mentions: Some(message_ids.to_vec()),
                },
            );
        }

        Ok(())
    }

    /// Fetch all channel unreads for a user.
    async fn fetch_unreads(&self, user_id: &str) -> Result<Vec<ChannelUnread>> {
        let unreads = self.channel_unreads.lock().await;
        Ok(unreads
            .values()
            .filter(|unread| unread.id.user == user_id)
            .cloned()
            .collect())
    }
}
