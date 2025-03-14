use mongodb::options::{Collation, CollationStrength, FindOneOptions};
use once_cell::sync::Lazy;

use crate::models::event::{Event, PartialEvent};
use crate::{AbstractEvents, Result};

use super::super::MongoDb;

static FIND_USERNAME_OPTIONS: Lazy<FindOneOptions> = Lazy::new(|| {
    FindOneOptions::builder()
        .collation(
            Collation::builder()
                .locale("en")
                .strength(CollationStrength::Secondary)
                .build(),
        )
        .build()
});

static COL: &str = "events";

#[async_trait]
impl AbstractEvents for MongoDb {
    async fn fetch_event(&self, id: &str) -> Result<Event> {
        self.find_one_by_id(COL, id).await
    }
    async fn insert_event(&self, event: &Event) -> Result<()> {
        self.insert_one(COL, event).await.map(|_| ())
    }

    async fn update_event(&self, id: &str, event: &PartialEvent) -> Result<()> {
        self.update_one_by_id(COL, id, event, vec![], None).await?;
        Ok(())
    }

    async fn delete_event(&self, id: &str) -> Result<()> {
        self.delete_one_by_id(COL, id).await.map(|_| ())
    }

    async fn fetch_events<'a>(&self, ids: &'a [String]) -> Result<Vec<Event>> {
        let filter = doc! {};

        self.find("events", filter).await
    }
}
