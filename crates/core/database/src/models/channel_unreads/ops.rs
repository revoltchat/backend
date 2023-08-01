use revolt_result::Result;

use crate::ChannelUnread;

mod mongodb;
mod reference;

#[async_trait]
pub trait AbstractChannelUnreads: Sync + Send {
    /// Acknowledge a message.
    async fn acknowledge_message(
        &self,
        channel_id: &str,
        user_id: &str,
        message_id: &str,
    ) -> Result<()>;

    /// Acknowledge many channels.
    async fn acknowledge_channels(&self, user_id: &str, channel_ids: &[String]) -> Result<()>;

    /// Add a mention.
    async fn add_mention_to_unread<'a>(
        &self,
        channel_id: &str,
        user_id: &str,
        message_ids: &[String],
    ) -> Result<()>;

    /// Fetch all channel unreads for a user.
    async fn fetch_unreads(&self, user_id: &str) -> Result<Vec<ChannelUnread>>;
}
