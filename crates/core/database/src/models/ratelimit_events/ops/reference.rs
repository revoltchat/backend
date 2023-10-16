use std::cmp::Ordering;
use std::time::Duration;
use std::time::SystemTime;

use super::AbstractRatelimitEvents;
use crate::RatelimitEvent;
use crate::RatelimitEventType;
use crate::ReferenceDb;
use revolt_result::Result;
use ulid::Ulid;

#[async_trait]
impl AbstractRatelimitEvents for ReferenceDb {
    /// Insert a new ratelimit event
    async fn insert_ratelimit_event(&self, event: &RatelimitEvent) -> Result<()> {
        let mut ratelimit_events = self.ratelimit_events.lock().await;
        if ratelimit_events.contains_key(&event.id) {
            Err(create_database_error!("insert", "message"))
        } else {
            ratelimit_events.insert(event.id.to_string(), event.clone());
            Ok(())
        }
    }

    /// Count number of events in given duration and check if we've hit the limit
    async fn has_ratelimited(
        &self,
        target_id: &str,
        event_type: RatelimitEventType,
        period: Duration,
        count: usize,
    ) -> Result<bool> {
        let ratelimit_events = self.ratelimit_events.lock().await;
        let gte_cmp_id = Ulid::from_datetime(SystemTime::now() - period).to_string();

        Ok(ratelimit_events
            .iter()
            .filter(|(id, event)| {
                id.cmp(&&gte_cmp_id) == Ordering::Greater
                    && event.target_id == target_id
                    && event.event_type == event_type
            })
            .count()
            >= count)
    }
}
