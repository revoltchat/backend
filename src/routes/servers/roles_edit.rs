use revolt_quark::{
    models::{
        server::{FieldsRole, PartialRole, Role},
        User,
    },
    perms, Db, Error, Ref, Result,
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
    if !perms(&user)
        .server(&server)
        .calc_server(db)
        .await
        .get_manage_roles()
    {
        return Err(Error::NotFound);
    }

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

        if let Some(remove) = &remove {
            for field in remove {
                role.remove(field);
            }
        }

        db.update_role(
            &server.id,
            &role_id,
            &partial,
            remove.unwrap_or_else(Vec::new),
        )
        .await?;

        role.apply_options(partial.clone());
        Ok(Json(role))
    } else {
        Err(Error::NotFound)
    }
}
