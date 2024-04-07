use revolt_database::{util::reference::Reference, Database};
use revolt_models::v0;
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};

/// # Fetch Role
///
/// Fetch a role by its id.
#[openapi(tag = "Server Permissions")]
#[get("/<target>/roles/<role_id>")]
pub async fn fetch(
    db: &State<Database>,
    target: Reference,
    role_id: String,
) -> Result<Json<v0::Role>> {
    let mut server = target.as_server(db).await?;
    let role = server.roles.remove(&role_id);

    if let Some(role) = role {
        Ok(Json(role.into()))
    } else {
        Err(create_error!(NotFound))
    }
}
