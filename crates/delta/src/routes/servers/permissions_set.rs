use rocket::serde::json::Json;
use serde::Deserialize;

use revolt_quark::{
    models::{Server, User},
    perms, Db, Error, Override, Permission, Ref, Result,
};

/// # Permission Value
#[derive(Deserialize, JsonSchema)]
pub struct DataSetServerRolePermission {
    /// Allow / deny values for the role in this server.
    permissions: Override,
}

/// # Set Role Permission
///
/// Sets permissions for the specified role in the server.
#[openapi(tag = "Server Permissions")]
#[put("/<target>/permissions/<role_id>", data = "<data>", rank = 2)]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    role_id: String,
    data: Json<DataSetServerRolePermission>,
) -> Result<Json<Server>> {
    let data = data.into_inner();

    let mut server = target.as_server(db).await?;
    if let Some((current_value, rank)) = server.roles.get(&role_id).map(|x| (x.permissions, x.rank))
    {
        let mut permissions = perms(&user).server(&server);

        permissions
            .throw_permission(db, Permission::ManagePermissions)
            .await?;

        if rank <= permissions.get_member_rank().unwrap_or(i64::MIN) {
            return Err(Error::NotElevated);
        }

        let current_value: Override = current_value.into();
        permissions
            .throw_permission_override(db, current_value, data.permissions)
            .await?;

        server
            .set_role_permission(db, &role_id, data.permissions.into())
            .await?;

        Ok(Json(server))
    } else {
        Err(Error::NotFound)
    }
}
