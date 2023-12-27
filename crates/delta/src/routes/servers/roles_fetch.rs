use revolt_quark::{models::server::Role, Db, Error, Ref, Result};
use rocket::serde::json::Json;

/// # Fetch Role
///
/// Fetch a role by its id.
#[openapi(tag = "Server Permissions")]
#[get("/<target>/roles/<role_id>")]
pub async fn req(db: &Db, target: Ref, role_id: String) -> Result<Json<Role>> {
    let server = target.as_server(db).await?;

    let role = server.roles.get(&role_id);

    if let Some(role) = role {
        Ok(Json(role.clone()))
    } else {
        Err(Error::NotFound)
    }
}
