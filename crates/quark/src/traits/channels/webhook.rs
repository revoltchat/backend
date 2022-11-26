use crate::models::webhook::{Webhook, PartialWebhook, FieldsWebhook};
use crate::Result;

#[async_trait]
pub trait AbstractWebhook: Sync + Send {
    async fn insert_webhook(&self, webhook: &Webhook) -> Result<()>;
    async fn fetch_webhook(&self, webhook_id: &str) -> Result<Webhook>;
    async fn delete_webhook(&self, webhook_id: &str) -> Result<()>;
    async fn update_webook(&self, webhook_id: &str, partial_webhook: &PartialWebhook, remove: &[FieldsWebhook]) -> Result<()>;
    async fn fetch_webhooks_for_channel(&self, channel: &str) -> Result<Vec<Webhook>>;
}
