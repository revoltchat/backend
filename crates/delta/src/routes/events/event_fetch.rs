use revolt_quark::{models::event::Event, Database, Result};
use rocket::{serde::json::Json, State};

/// Get event by id
#[openapi(tag = "Events")]
#[get("/<id>")]
pub async fn get_event(db: &State<Database>, id: String) -> Result<Json<Event>> {
    let event = db.fetch_event(&id).await?;
    Ok(Json(event))
}
