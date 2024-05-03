use std::time::Duration;

use super::AbstractRatelimitEvents;
use crate::RatelimitEvent;
use crate::RatelimitEventType;
use crate::ReferenceDb;
use revolt_result::Result;

#[async_trait]
impl AbstractRatelimitEvents for ReferenceDb {
    /// Insert a new ratelimit event
    async fn insert_ratelimit_event(&self, _event: &RatelimitEvent) -> Result<()> {
        // TODO: implement
        unimplemented!()
    }

    /// Count number of events in given duration and check if we've hit the limit
    async fn has_ratelimited(
        &self,
        _target_id: &str,
        _event_type: RatelimitEventType,
        _period: Duration,
        _count: usize,
    ) -> Result<bool> {
        // TODO: implement
        unimplemented!()
    }
}
