use revolt_result::Result;

use crate::ReferenceDb;
use crate::{FieldsWebhook, PartialWebhook, Webhook};

use super::AbstractWebhooks;

#[async_trait]
impl AbstractWebhooks for ReferenceDb {
    /// Insert new webhook into the database
    async fn insert_webhook(&self, webhook: &Webhook) -> Result<()> {
        let mut webhooks = self.channel_webhooks.lock().await;
        if webhooks.contains_key(&webhook.id) {
            Err(create_database_error!("insert", "webhook"))
        } else {
            webhooks.insert(webhook.id.to_string(), webhook.clone());
            Ok(())
        }
    }

    /// Fetch webhook by id
    async fn fetch_webhook(&self, webhook_id: &str) -> Result<Webhook> {
        let webhooks = self.channel_webhooks.lock().await;
        webhooks
            .get(webhook_id)
            .cloned()
            .ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch webhooks for channel
    async fn fetch_webhooks_for_channel(&self, channel_id: &str) -> Result<Vec<Webhook>> {
        let webhooks = self.channel_webhooks.lock().await;
        Ok(webhooks
            .values()
            .filter(|webhook| webhook.channel_id == channel_id)
            .cloned()
            .collect())
    }

    /// Update webhook with new information
    async fn update_webhook(
        &self,
        webhook_id: &str,
        partial: &PartialWebhook,
        remove: &[FieldsWebhook],
    ) -> Result<()> {
        let mut webhooks = self.channel_webhooks.lock().await;
        if let Some(webhook) = webhooks.get_mut(webhook_id) {
            for field in remove {
                #[allow(clippy::disallowed_methods)]
                webhook.remove_field(field);
            }

            webhook.apply_options(partial.clone());
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }

    /// Delete webhook by id
    async fn delete_webhook(&self, webhook_id: &str) -> Result<()> {
        let mut webhooks = self.channel_webhooks.lock().await;
        if webhooks.remove(webhook_id).is_some() {
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }
}
