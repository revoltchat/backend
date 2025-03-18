use crate::models::event::{Event, PartialEvent};
use crate::Result;

#[async_trait]
pub trait AbstractEvents: Sync + Send {
    async fn fetch_event(&self, user_id: Option<&str>, id: &str) -> Result<Event>;
    async fn fetch_events<'a>(
        &self,
        user_id: Option<&str>,
        ids: &'a [String],
    ) -> Result<Vec<Event>>;
    async fn insert_event(&self, event: &Event) -> Result<()>;
    async fn update_event(&self, id: &str, event: &PartialEvent) -> Result<()>;
    async fn delete_event(&self, id: &str) -> Result<()>;
    async fn toggle_saved_event(&self, user_id: &str, event_id: &str) -> Result<(Event, bool)>;
    async fn is_event_saved(&self, user_id: &str, event_id: &str) -> Result<bool>;
    async fn get_saved_events(&self, user_id: &str) -> Result<Vec<Event>>;
    /// Get all events created by a user
    async fn get_user_events(&self, user_id: &str) -> Result<Vec<Event>>;
}
