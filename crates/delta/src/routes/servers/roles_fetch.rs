use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    Database, User,
};
use revolt_models::v0;
use revolt_permissions::PermissionQuery;
use revolt_result::{create_error, Result};
use rocket::State;
use crate::util::json::Json;

/// # Fetch Role
///
/// Fetch a role by its id.
#[openapi(tag = "Server Permissions")]
#[get("/<target>/roles/<role_id>")]
pub async fn fetch(
    db: &State<Database>,
    user: User,
    target: Reference,
    role_id: String,
) -> Result<Json<v0::Role>> {
    let mut server = target.as_server(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    if !query.are_we_a_member().await {
        return Err(create_error!(NotFound));
    }

    let role = server.roles.remove(&role_id);

    if let Some(role) = role {
        Ok(Json(role.into()))
    } else {
        Err(create_error!(NotFound))
    }
}
