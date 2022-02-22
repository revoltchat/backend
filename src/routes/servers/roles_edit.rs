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
    perms(&user)
        .server(&server)
        .throw_permission(db, Permission::ManageRole)
        .await?;

    if let Some(mut role) = server.roles.remove(&role_id) {
        let Data {
            name,
            colour,
            hoist,
            rank,
            remove,
        } = data;

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
