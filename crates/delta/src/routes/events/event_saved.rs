use revolt_quark::models::event::Event;
use revolt_quark::models::user::User;
use revolt_quark::{Database, Result};
use rocket::{serde::json::Json, State};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct EventSaveResponse {
    event: Event,
    saved: bool,
}

/// Toggle saved status for an event
#[openapi(tag = "Events")]
#[post("/<event_id>/save")]
pub async fn toggle_saved_event(
    db: &State<Database>,
    user: User,
    event_id: String,
) -> Result<Json<EventSaveResponse>> {
    let (event, is_saved) = db.toggle_saved_event(&user.id, &event_id).await?;
    Ok(Json(EventSaveResponse {
        event,
        saved: is_saved,
    }))
}

/// Get user's saved events
#[openapi(tag = "Events")]
#[get("/saved")]
pub async fn get_saved_events(db: &State<Database>, user: User) -> Result<Json<Vec<Event>>> {
    let events = db.get_saved_events(&user.id).await?;
    Ok(Json(events))
}
