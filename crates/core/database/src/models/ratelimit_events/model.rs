use std::fmt;

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
