use revolt_quark::{
    models::{server::Role, User},
    perms, Db, Error, Permission, Ref, Result,
};

use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// # Role Data
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataCreateRole {
    /// Role name
    #[validate(length(min = 1, max = 32))]
    name: String,
    /// Ranking position
    ///
    /// Smaller values take priority.
    rank: Option<i64>,
}

/// # Create Role
///
/// Creates a new server role.
#[openapi(tag = "Server Permissions")]
#[post("/<target>/roles", data = "<data>")]
pub async fn req(db: &Db, user: User, target: Ref, data: Json<DataCreateRole>) -> Result<Value> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let server = target.as_server(db).await?;
    let mut permissions = perms(&user).server(&server);

    permissions
        .throw_permission(db, Permission::ManageRole)
        .await?;

    let member_rank = permissions.get_member_rank();
    let rank = if let Some(given_rank) = data.rank {
        if given_rank <= member_rank.unwrap_or(i64::MIN) {
            return Err(Error::NotElevated);
        }

        given_rank
    } else {
        member_rank.unwrap_or(0).saturating_add(1)
    };

    let role = Role {
        name: data.name,
        rank,
        ..Default::default()
    };

    Ok(json!({
        "id": role.create(db, &server.id).await?,
        "role": role
    }))
}
