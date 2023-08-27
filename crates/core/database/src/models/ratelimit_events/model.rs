use std::fmt;

use revolt_result::Result;
use ulid::Ulid;

use crate::Database;

auto_derived!(
    /// Ratelimit Event
    pub struct RatelimitEvent {
        /// Id
        #[serde(rename = "_id")]
        pub id: String,
        /// Relevant Object Id
        pub target_id: String,
        /// Type of event
        pub event_type: RatelimitEventType,
    }

    /// Event type
    pub enum RatelimitEventType {
        DiscriminatorChange,
    }
);

impl fmt::Display for RatelimitEventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[allow(clippy::disallowed_methods)]
impl RatelimitEvent {
    /// Create ratelimit event
    pub async fn create(
        db: &Database,
        target_id: String,
        event_type: RatelimitEventType,
    ) -> Result<()> {
        db.insert_ratelimit_event(&RatelimitEvent {
            id: Ulid::new().to_string(),
            target_id,
            event_type,
        })
        .await
    }
}
