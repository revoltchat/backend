use revolt_quark::models::event::{Event, EventType};
use revolt_quark::{Database, Result};
use rocket::{http::uri::Query, serde::json::Json, State};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct EventQuery {
    pub event_type: Option<EventType>,
    pub start_date_from: Option<String>,
    pub start_date_to: Option<String>,
    pub city: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// List all events with optional filtering
#[openapi(tag = "Events")]
#[get("/")]
pub async fn list_events(db: &State<Database>) -> Result<Json<Vec<Event>>> {
    let events = db.fetch_events(&[]).await?;
    Ok(Json(events))
}
