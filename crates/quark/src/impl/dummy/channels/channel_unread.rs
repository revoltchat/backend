use crate::models::channel_unread::ChannelUnread;
use crate::{AbstractChannelUnread, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractChannelUnread for DummyDb {
    async fn acknowledge_message(&self, channel: &str, user: &str, message: &str) -> Result<()> {
        info!("Acknowledged {message} in {channel} for {user}");
        Ok(())
    }

    async fn acknowledge_channels(&self, user: &str, channels: &[String]) -> Result<()> {
        info!("Acknowledged {channels:?} for {user}");
        Ok(())
    }

    async fn add_mention_to_unread<'a>(
        &self,
        channel: &str,
        user: &str,
        ids: &[String],
    ) -> Result<()> {
        info!("Added mentions for {user} in {channel}: {ids:?}");
        Ok(())
    }

    async fn fetch_unreads(&self, _user: &str) -> Result<Vec<ChannelUnread>> {
        Ok(vec![])
    }
}
