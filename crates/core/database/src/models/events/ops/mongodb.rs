use super::AbstractEvents;
use crate::{Event, FieldsEvent, IntoDocumentPath, MongoDb, PartialEvent};
use futures::StreamExt;
use mongodb::bson::doc;
use revolt_result::Result;

static COL: &str = "events";

#[async_trait]
impl AbstractEvents for MongoDb {
    async fn fetch_event(&self, id: &str) -> Result<Event> {
        query!(self, find_one_by_id, COL, id)?.ok_or_else(|| create_error!(NotFound))
    }

    async fn fetch_events<'a>(&self, ids: &'a [String]) -> Result<Vec<Event>> {
        Ok(self
            .col::<Event>(COL)
            .find(
                doc! {
                    "_id": {
                        "$in": ids
                    }
                },
                None,
            )
            .await
            .map_err(|_| create_database_error!("fetch", "channels"))?
            .filter_map(|s| async {
                if cfg!(debug_assertions) {
                    Some(s.unwrap())
                } else {
                    s.ok()
                }
            })
            .collect()
            .await)
    }

    async fn insert_event(&self, event: &Event) -> Result<()> {
        query!(self, insert_one, COL, event).map(|_| ())
    }

    async fn update_event(
        &self,
        id: &str,
        event: &PartialEvent,
        remove: Vec<FieldsEvent>,
    ) -> Result<()> {
        query!(
            self,
            update_one_by_id,
            COL,
            id,
            event,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            None
        )
        .map(|_| ())
    }

    async fn delete_event(&self, id: &str) -> Result<()> {
        query!(self, delete_one_by_id, COL, id).map(|_| ())
    }
}
