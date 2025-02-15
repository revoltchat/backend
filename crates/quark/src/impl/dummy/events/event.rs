use crate::models::event::{Event, PartialEvent};
use crate::{AbstractEvents, Error, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractEvents for DummyDb {
    async fn insert_event(&self, _: &Event) -> Result<()> {
        Ok(())
    }

    async fn fetch_event(&self, _: &str) -> Result<Event> {
        Err(Error::NotFound)
    }
    async fn fetch_events<'a>(&self, ids: &'a [String]) -> Result<Vec<Event>> {
        Ok(vec![])
    }

    async fn update_event(&self, _: &str, _: &PartialEvent) -> Result<()> {
        Ok(())
    }

    async fn delete_event(&self, _: &str) -> Result<()> {
        Ok(())
    }
}
