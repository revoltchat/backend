use nanoid::nanoid;

use crate::{
    models::{bot::FieldsBot, Bot},
    Database, Result,
};

impl Bot {
    /// Remove a field from this object
    pub fn remove(&mut self, field: &FieldsBot) {
        match field {
            FieldsBot::Token => self.token = nanoid!(64),
            FieldsBot::InteractionsURL => {
                self.interactions_url.take();
            }
        }
    }

    /// Delete this bot
    pub async fn delete(&self, db: &Database) -> Result<()> {
        db.fetch_user(&self.id).await?.mark_deleted(db).await?;
        db.delete_bot(&self.id).await
    }
}
