use revolt_quark::{models::event::Event, models::user::User, Database, Result};
use rocket::{serde::json::Json, State};

/// Get event by id
#[openapi(tag = "Events")]
#[get("/<id>")]
pub async fn get_event(
    db: &State<Database>,
    user: Option<User>,
    id: String,
) -> Result<Json<Event>> {
    let event = db
        .fetch_event(user.as_ref().map(|u| u.id.as_str()), &id)
        .await?;
    Ok(Json(event))
}
