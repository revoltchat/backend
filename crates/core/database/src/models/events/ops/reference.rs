use super::AbstractEvents;
use crate::ReferenceDb;
use crate::{Event, FieldsEvent, PartialEvent};
use revolt_result::Result;

#[async_trait]
impl AbstractEvents for ReferenceDb {
    async fn fetch_event(&self, _: &str) -> Result<Event> {
        Err(create_error!(NotFound))
    }

    async fn fetch_events<'a>(&self, ids: &'a [String]) -> Result<Vec<Event>> {
        let events = self.events.lock().await;
        ids.iter()
            .map(|id| {
                events
                    .get(id)
                    .cloned()
                    .ok_or_else(|| create_error!(NotFound))
            })
            .collect()
    }

    async fn insert_event(&self, _: &Event) -> Result<()> {
        Ok(())
    }

    async fn update_event(&self, _: &str, _: &PartialEvent, _: Vec<FieldsEvent>) -> Result<()> {
        Ok(())
    }

    async fn delete_event(&self, _: &str) -> Result<()> {
        Ok(())
    }
}
