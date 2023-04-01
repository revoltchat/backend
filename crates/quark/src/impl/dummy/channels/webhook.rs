use crate::models::webhook::{Webhook, FieldsWebhook, PartialWebhook};
use crate::{AbstractWebhook, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractWebhook for DummyDb {
    async fn insert_webhook(&self, webhook: &Webhook) -> Result<()> {
        info!("Created webhook {} in {}", webhook.name, webhook.channel);
        Ok(())
    }

    async fn delete_webhook(&self, webhook_id: &str) -> Result<()> {
        info!("deleting webhook {webhook_id}");

        Ok(())
    }

    async fn fetch_webhook(&self, webhook_id: &str) -> Result<Webhook> {
        Ok(Webhook {
            id: webhook_id.to_string(),
            name: "".to_string(),
            avatar: None,
            channel: "0".to_string(),
            token: Some("".to_owned()),
        })
    }

    async fn update_webook(&self, webhook_id: &str, partial_webhook: &PartialWebhook, remove: &[FieldsWebhook]) -> Result<()> {
        info!("updating webhook {webhook_id}, {partial_webhook:?}, removing {remove:?}");

        Ok(())
    }

    async fn fetch_webhooks_for_channel(&self, channel: &str) -> Result<Vec<Webhook>> {
        info!("fetching webhooks for channel {channel}");

        Ok(vec![self.fetch_webhook("0").await?])
    }
}
