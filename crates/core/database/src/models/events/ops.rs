use crate::models::events::{Event, FieldsEvent, PartialEvent};
use crate::revolt_result::Result;
mod mongodb;
mod reference;

#[async_trait]
pub trait AbstractEvents: Sync + Send {
    async fn fetch_event(&self, _: &str) -> Result<Event>;

    async fn fetch_events<'a>(&self, ids: &'a [String]) -> Result<Vec<Event>>;

    async fn insert_event(&self, _: &Event) -> Result<()>;

    async fn update_event(&self, _: &str, _: &PartialEvent, _: Vec<FieldsEvent>) -> Result<()>;

    async fn delete_event(&self, _: &str) -> Result<()>;
}
