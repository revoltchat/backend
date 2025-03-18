use revolt_quark::models::event::{Event, EventType};
use revolt_quark::models::user::User;
use revolt_quark::{Database, Result};
use rocket::{serde::json::Json, State};
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
pub async fn list_events(db: &State<Database>, user: Option<User>) -> Result<Json<Vec<Event>>> {
    let events = db
        .fetch_events(user.as_ref().map(|u| u.id.as_str()), &[])
        .await?;
    Ok(Json(events))
}

/// Get all events created by the current user
#[openapi(tag = "Events")]
#[get("/created")]
pub async fn get_created_events(db: &State<Database>, user: User) -> Result<Json<Vec<Event>>> {
    let events = db.get_user_events(&user.id).await?;
    Ok(Json(events))
}
