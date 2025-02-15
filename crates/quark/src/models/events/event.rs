use serde::{Deserialize, Serialize};

pub fn if_false(t: &bool) -> bool {
    !t
}

/// Representation of an event
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, OptionalStruct, Default)]
#[optional_derive(Serialize, Deserialize, JsonSchema, Debug, Default, Clone)]
#[optional_name = "PartialEvent"]
#[opt_skip_serializing_none]
#[opt_some_priority]
pub struct Event {
    /// Event Id
    #[serde(rename = "_id")]
    pub id: String,

    /// Event title
    pub title: String,

    /// Event type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<EventType>,

    /// Start date and time
    pub start_date: String,

    /// End date and time
    pub end_date: String,

    /// City where event is held
    pub city: String,

    /// Whether the event is public or private
    #[serde(skip_serializing_if = "if_false", default)]
    pub hide_address: bool,

    /// Area/neighborhood
    pub area: String,

    /// Full address
    pub address: String,

    /// Event description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Allow +1 guests
    #[serde(skip_serializing_if = "if_false", default)]
    pub allow_plus_one: bool,

    /// Require full information for +1 guests
    #[serde(skip_serializing_if = "if_false", default)]
    pub requires_plus_one_info: bool,

    /// Require RSVP approval by host
    #[serde(skip_serializing_if = "if_false", default)]
    pub requires_rsvp_approval: bool,

    /// Show events to non-members
    #[serde(skip_serializing_if = "if_false", default)]
    pub show_to_non_members: bool,

    /// Event managers (user IDs)
    pub managers: Vec<String>,

    /// Event sponsors (user IDs)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub sponsors: Vec<String>,

    /// Ticket configuration
    pub ticket_config: TicketConfig,

    /// Attachment URLs
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub attachments: Vec<String>,

    /// Gallery image URLs
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub gallery: Vec<String>,

    /// Creation timestamp
    pub created_at: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub enum EventType {
    KimaniEvent,
    MembersEvent,
    Other,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default)]
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
