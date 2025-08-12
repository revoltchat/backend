use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    Database, User,
};
use revolt_models::v0;
use revolt_permissions::{calculate_server_permissions, ChannelPermission, Override};
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};

/// # Set Role Permission
///
/// Sets permissions for the specified role in the server.
#[openapi(tag = "Server Permissions")]
#[put("/<target>/permissions/<role_id>", data = "<data>", rank = 2)]
pub async fn set_role_permission(
    db: &State<Database>,
    user: User,
    target: Reference<'_>,
    role_id: String,
    data: Json<v0::DataSetServerRolePermission>,
) -> Result<Json<v0::Server>> {
    let data = data.into_inner();

    let mut server = target.as_server(db).await?;
    if let Some((current_value, rank)) = server.roles.get(&role_id).map(|x| (x.permissions, x.rank))
    {
        let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
        let permissions = calculate_server_permissions(&mut query).await;

        permissions.throw_if_lacking_channel_permission(ChannelPermission::ManagePermissions)?;

        // Prevent us from editing roles above us
        if rank <= query.get_member_rank().unwrap_or(i64::MIN) {
            return Err(create_error!(NotElevated));
        }

        // Ensure we have access to grant these permissions forwards
        let current_value: Override = current_value.into();
        permissions
            .throw_permission_override(current_value, &data.permissions)
            .await?;

        server
            .set_role_permission(db, &role_id, data.permissions.into())
            .await?;

        Ok(Json(server.into()))
    } else {
        Err(create_error!(NotFound))
    }
}
