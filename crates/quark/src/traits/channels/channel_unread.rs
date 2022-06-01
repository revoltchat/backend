use crate::models::channel_unread::ChannelUnread;
use crate::Result;

#[async_trait]
pub trait AbstractChannelUnread: Sync + Send {
    /// Acknowledge a message.
    async fn acknowledge_message(&self, channel: &str, user: &str, message: &str) -> Result<()>;

    /// Acknowledge many channels.
    async fn acknowledge_channels(&self, user: &str, channels: &[String]) -> Result<()>;

    /// Add a mention.
    async fn add_mention_to_unread<'a>(
        &self,
        channel: &str,
        user: &str,
        ids: &[String],
    ) -> Result<()>;

    /// Fetch all channel unreads for a user.
    async fn fetch_unreads(&self, user: &str) -> Result<Vec<ChannelUnread>>;
}
