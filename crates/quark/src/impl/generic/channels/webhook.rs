use crate::{
    events::client::EventV1,
    models::{
        webhook::{Webhook, PartialWebhook, FieldsWebhook},
        message::MessageWebhook
    },
    Database, Result
};

impl Webhook {
    pub async fn create(&self, db: &Database) -> Result<()> {
        db.insert_webhook(self).await?;

        // Avoid leaking the token to people who receive the event
        let mut webhook = self.clone();
        webhook.token = None;

        EventV1::WebhookCreate(webhook)
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
        mut partial: PartialWebhook,
        remove: Vec<FieldsWebhook>
    ) -> Result<()> {
        for field in &remove {
            self.remove(field)
        };

        self.apply_options(partial.clone());

        db.update_webook(&self.id, &partial, &remove).await?;

        partial.token = None;  // Avoid leaking the token to people who receive the event

        EventV1::WebhookUpdate {
            id: self.id.clone(),
            data: partial,
            remove
        }
            .p(self.channel.clone())
            .await;

        Ok(())
    }

    pub fn remove(&mut self, field: &FieldsWebhook) {
        match field {
            FieldsWebhook::Avatar => self.avatar = None
        }
    }

    pub fn into_message_webhook(self) -> MessageWebhook {
        MessageWebhook {
            id: self.id,
            name: self.name,
            avatar: self.avatar.map(|f| f.id)
        }
    }
}
