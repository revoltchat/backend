use rocket::serde::json::Json;
use serde::Deserialize;

use revolt_quark::{
    models::{server::PartialServer, Server, User},
    perms, Db, Permission, Ref, Result,
};

/// # Permission Value
#[derive(Deserialize, JsonSchema)]
pub struct DataSetServerDefaultPermission {
    /// Default member permission value
    permissions: u64,
}

/// # Set Default Permission
///
/// Sets permissions for the default role in this server.
#[openapi(tag = "Server Permissions")]
#[put("/<target>/permissions/default", data = "<data>", rank = 1)]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    data: Json<DataSetServerDefaultPermission>,
) -> Result<Json<Server>> {
    let data = data.into_inner();

    let mut server = target.as_server(db).await?;
    let mut permissions = perms(&user).server(&server);

    permissions
        .throw_permission(db, Permission::ManagePermissions)
        .await?;

    permissions
        .throw_permission_value(db, data.permissions)
        .await?;

    server
        .update(
            db,
            PartialServer {
                default_permissions: Some(data.permissions as i64),
                ..Default::default()
            },
            vec![],
        )
        .await?;

    Ok(Json(server))
}
