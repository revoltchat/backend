use crate::{
    events::client::EventV1,
    models::{
        webhook::{Webhook, PartialWebhook, FieldsWebhook}
    },
    Database, Result
};

impl Webhook {
    pub async fn create(&self, db: &Database) -> Result<()> {
        db.insert_webhook(self).await?;

        EventV1::WebhookCreate(self.clone())
            .p(self.channel.clone()).await;

        Ok(())
    }

    pub async fn delete(&self, db: &Database) -> Result<()> {
        db.delete_webhook(&self.id).await?;

        EventV1::WebhookDelete { id: self.id.clone() }
            .p(self.channel.clone()).await;

        Ok(())
    }

    pub async fn update(
        &mut self,
        db: &Database,
        partial: PartialWebhook,
        remove: Vec<FieldsWebhook>
    ) -> Result<()> {
        for field in &remove {
            self.remove(field)
        };

        self.apply_options(partial.clone());

        db.update_webook(&self.id, &partial, &remove).await?;

        EventV1::WebhookUpdate {
            id: self.id.clone(),
            data: partial,
            remove
        }
            .p(self.channel.clone())
            .await;

        Ok(())
    }

    fn remove(&mut self, field: &FieldsWebhook) {
        match field {
            FieldsWebhook::Avatar => self.avatar = None
        }
    }
}
