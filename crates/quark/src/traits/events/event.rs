use crate::models::event::{Event, PartialEvent};
use crate::Result;

#[async_trait]
pub trait AbstractEvents: Sync + Send {
    async fn fetch_event(&self, id: &str) -> Result<Event>;
    async fn fetch_events<'a>(&self, ids: &'a [String]) -> Result<Vec<Event>>;
    async fn insert_event(&self, event: &Event) -> Result<()>;
    async fn update_event(&self, id: &str, event: &PartialEvent) -> Result<()>;
    async fn delete_event(&self, id: &str) -> Result<()>;
}
