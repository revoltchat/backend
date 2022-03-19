use rocket::serde::json::Json;
use serde::Deserialize;

use revolt_quark::{
    models::{Channel, User},
    perms, Db, Error, Override, Permission, Ref, Result,
};

/// # Permission Value
#[derive(Deserialize, JsonSchema)]
pub struct Data {
    /// Allow / deny values to set for this role
    permissions: Override,
}

/// # Set Role Permission
///
/// Sets permissions for the specified role in this channel.
///
/// Channel must be a `TextChannel` or `VoiceChannel`.
#[openapi(tag = "Channel Permissions")]
#[put("/<target>/permissions/<role_id>", data = "<data>", rank = 2)]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    role_id: String,
    data: Json<Data>,
) -> Result<Json<Channel>> {
    let mut channel = target.as_channel(db).await?;
    let mut permissions = perms(&user).channel(&channel);

    permissions
        .throw_permission_and_view_channel(db, Permission::ManagePermissions)
        .await?;

    if let Some(server) = permissions.server.get() {
        if let Some(role) = server.roles.get(&role_id) {
            if role.rank <= permissions.get_member_rank().unwrap_or(i64::MIN) {
                return Err(Error::NotElevated);
            }

            let current_value: Override = role.permissions.into();
            permissions
                .throw_permission_override(db, current_value, data.permissions)
                .await?;

            channel
                .set_role_permission(db, &role_id, data.permissions.into())
                .await?;

            Ok(Json(channel))
        } else {
            Err(Error::NotFound)
        }
    } else {
        Err(Error::InvalidOperation)
    }
}
