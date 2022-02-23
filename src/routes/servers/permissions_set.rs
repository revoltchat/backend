use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use revolt_quark::{
    models::{Server, User},
    perms, Db, Error, Override, Permission, Ref, Result,
};

#[derive(Serialize, Deserialize)]
pub struct Data {
    permissions: Override,
}

#[put("/<target>/permissions/<role_id>", data = "<data>", rank = 2)]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    role_id: String,
    data: Json<Data>,
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

        if !permissions
            .has_permission_value(db, data.permissions.allows())
            .await?
        {
            return Err(Error::CannotGiveMissingPermissions);
        }

        let current_value: Override = current_value.into();
        if !permissions
            .has_permission_value(db, current_value.denies() & (!data.permissions.denies()))
            .await?
        {
            return Err(Error::CannotGiveMissingPermissions);
        }

        server
            .set_role_permission(db, &role_id, data.permissions.into())
            .await?;

        Ok(Json(server))
    } else {
        Err(Error::NotFound)
    }
}
