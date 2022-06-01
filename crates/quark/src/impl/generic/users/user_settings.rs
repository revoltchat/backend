use crate::{events::client::EventV1, models::UserSettings, Database, Result};

#[async_trait]
pub trait UserSettingsImpl {
    async fn set(self, db: &Database, user: &str) -> Result<()>;
}

#[async_trait]
impl UserSettingsImpl for UserSettings {
    async fn set(self, db: &Database, user: &str) -> Result<()> {
        db.set_user_settings(user, &self).await?;

        EventV1::UserSettingsUpdate {
            id: user.to_string(),
            update: self,
        }
        .private(user.to_string())
        .await;

        Ok(())
    }
}
