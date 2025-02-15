use revolt_quark::{Database, Result};
use rocket::State;

/// Delete event
#[openapi(tag = "Events")]
#[delete("/<id>")]
pub async fn delete_event(db: &State<Database>, id: String) -> Result<()> {
    db.delete_event(&id).await?;
    Ok(())
}
