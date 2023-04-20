use crate::models::{webhook::{Webhook, PartialWebhook, FieldsWebhook}};
use crate::r#impl::mongo::IntoDocumentPath;
use crate::{Result, AbstractWebhook};

use super::super::MongoDb;

static COL: &str = "webhooks";

#[async_trait]
impl AbstractWebhook for MongoDb {
    async fn insert_webhook(&self, webhook: &Webhook) -> Result<()> {
        self.insert_one(COL, webhook).await?;

        Ok(())
    }

    async fn fetch_webhook(&self, webhook_id: &str) -> Result<Webhook> {
        info!("{COL} {webhook_id}");

        self.find_one_by_id(COL, webhook_id).await
    }

    async fn delete_webhook(&self, webhook_id: &str) -> Result<()> {
        self.delete_one_by_id(COL, webhook_id).await?;

        Ok(())
    }

    async fn update_webhook(&self, webhook_id: &str, partial_webhook: &PartialWebhook, remove: &[FieldsWebhook]) -> Result<()> {
        self.update_one_by_id(
            COL,
            webhook_id,
            partial_webhook,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            None,
        )
        .await
        .map(|_| ())
    }

    async fn fetch_webhooks_for_channel(&self, channel: &str) -> Result<Vec<Webhook>> {
        self.find(
            COL,
            doc! {
                "channel": channel
        })
            .await
    }
}

impl IntoDocumentPath for FieldsWebhook {
    fn as_path(&self) -> Option<&'static str> {
        match self {
            FieldsWebhook::Avatar => Some("avatar"),
        }
    }
}
