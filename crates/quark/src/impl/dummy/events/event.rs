use crate::models::event::{Event, PartialEvent};
use crate::{AbstractEvents, Error, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractEvents for DummyDb {
    async fn insert_event(&self, _: &Event) -> Result<()> {
        Ok(())
    }

    async fn fetch_event(&self, _: Option<&str>, _: &str) -> Result<Event> {
        Err(Error::NotFound)
    }
    async fn fetch_events<'a>(&self, _: Option<&str>, _: &'a [String]) -> Result<Vec<Event>> {
        Ok(vec![])
    }

    async fn update_event(&self, _: &str, _: &PartialEvent) -> Result<()> {
        Ok(())
    }

    async fn delete_event(&self, _: &str) -> Result<()> {
        Ok(())
    }

    async fn toggle_saved_event(&self, _: &str, _: &str) -> Result<(Event, bool)> {
        Ok((Event::default(), true))
    }

    async fn is_event_saved(&self, _: &str, _: &str) -> Result<bool> {
        Ok(false)
    }

    async fn get_saved_events(&self, _: &str) -> Result<Vec<Event>> {
        Ok(vec![])
    }

    async fn get_user_events(&self, _: &str) -> Result<Vec<Event>> {
        Ok(vec![])
    }
}
