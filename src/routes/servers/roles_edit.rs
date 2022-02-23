use revolt_quark::{
    models::{
        server::{FieldsRole, PartialRole, Role},
        User,
    },
    perms, Db, Error, Permission, Ref, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    name: Option<String>,
    #[validate(length(min = 1, max = 32))]
    colour: Option<String>,
    hoist: Option<bool>,
    rank: Option<i64>,
    #[validate(length(min = 1))]
    remove: Option<Vec<FieldsRole>>,
}

#[patch("/<target>/roles/<role_id>", data = "<data>")]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    role_id: String,
    data: Json<Data>,
) -> Result<Json<Role>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let mut server = target.as_server(db).await?;
    let mut permissions = perms(&user).server(&server);

    permissions
        .throw_permission(db, Permission::ManageRole)
        .await?;

    let member_rank = permissions.get_member_rank().unwrap_or(i64::MIN);

    if let Some(mut role) = server.roles.remove(&role_id) {
        let Data {
            name,
            colour,
            hoist,
            rank,
            remove,
        } = data;

        if let Some(rank) = &rank {
            if rank <= &member_rank {
                return Err(Error::NotElevated);
            }
        }

        let partial = PartialRole {
            name,
            colour,
            hoist,
            rank,
            ..Default::default()
        };

        role.update(
            db,
            &server.id,
            &role_id,
            partial,
            remove.unwrap_or_default(),
        )
        .await?;

        Ok(Json(role))
    } else {
        Err(Error::NotFound)
    }
}
