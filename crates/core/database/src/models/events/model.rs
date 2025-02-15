use crate::IntoDocumentPath;

auto_derived!(
    /// Event Type
    pub enum EventType {
        KimaniEvent,
        MembersEvent,
        Other,
    }

    /// Ticket Configuration
    pub struct TicketConfig {
        /// Type of ticket (free or paid)
        pub is_paid: bool,
        /// Member ticket price (if paid)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub member_price: Option<String>,
        /// Maximum tickets for members
        #[serde(skip_serializing_if = "Option::is_none")]
        pub member_max_tickets: Option<i32>,
        /// Non-member ticket price (if paid)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub non_member_price: Option<String>,
        /// Maximum tickets for non-members
        #[serde(skip_serializing_if = "Option::is_none")]
        pub non_member_max_tickets: Option<i32>,
        /// Allow purchase of multiple tickets
        pub allow_multiple_tickets: bool,
        /// Processing fee percentage
        pub processing_fee_percentage: Option<String>,
    }

    /// Query for events
    pub struct EventQuery {
        /// Event type
        pub event_type: Option<EventType>,
        /// Start date from
        pub start_date_from: Option<String>,
        /// Start date to
        pub start_date_to: Option<String>,
        /// City
        pub city: Option<String>,
        /// Page number
        pub page: Option<i64>,
        /// Results per page
        pub per_page: Option<i64>,
    }
);

impl Default for EventType {
    fn default() -> Self {
        Self::Other
    }
}

impl Default for TicketConfig {
    fn default() -> Self {
        Self {
            is_paid: false,
            member_price: None,
            member_max_tickets: None,
            non_member_price: None,
            non_member_max_tickets: None,
            allow_multiple_tickets: false,
            processing_fee_percentage: None,
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::KimaniEvent => write!(f, "KimaniEvent"),
            EventType::MembersEvent => write!(f, "MembersEvent"),
            EventType::Other => write!(f, "Other"),
        }
    }
}

auto_derived_partial!(
    /// Event
    pub struct Event {
        /// Event Id
        #[serde(rename = "_id")]
        pub id: String,

        /// Event title
        pub title: String,

        /// Event type
        pub event_type: EventType,

        /// Start date and time
        pub start_date: String,

        /// End date and time
        pub end_date: String,

        /// City where event is held
        pub city: String,

        /// Whether to hide the address
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub hide_address: bool,

        /// Area/neighborhood
        pub area: String,

        /// Full address
        pub address: String,

        /// Event description
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,

        /// Allow +1 guests
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub allow_plus_one: bool,

        /// Require full information for +1 guests
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub requires_plus_one_info: bool,

        /// Require RSVP approval by host
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub requires_rsvp_approval: bool,

        /// Show events to non-members
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub show_to_non_members: bool,

        /// Event managers (user IDs)
        pub managers: Vec<String>,

        /// Event sponsors (user IDs)
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub sponsors: Vec<String>,

        /// Ticket configuration
        pub ticket_config: TicketConfig,

        /// Creation timestamp
        pub created_at: String,
    },
    "PartialEvent"
);

auto_derived!(
    /// Optional fields on event object
    pub enum FieldsEvent {
        Description,
        Managers,
        Sponsors,
        TicketConfig,
    }
);

impl IntoDocumentPath for FieldsEvent {
    fn as_path(&self) -> Option<&'static str> {
        match self {
            FieldsEvent::Description => "description".into(),
            FieldsEvent::Managers => "managers".into(),
            FieldsEvent::Sponsors => "sponsors".into(),
            FieldsEvent::TicketConfig => "ticket_config".into(),
        }
    }
}
