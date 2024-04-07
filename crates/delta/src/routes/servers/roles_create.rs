use revolt_config::config;
use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    Database, Role, User,
};
use revolt_models::v0;
use revolt_permissions::{calculate_server_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};
use validator::Validate;

/// # Create Role
///
/// Creates a new server role.
#[openapi(tag = "Server Permissions")]
#[post("/<target>/roles", data = "<data>")]
pub async fn create(
    db: &State<Database>,
    user: User,
    target: Reference,
    data: Json<v0::DataCreateRole>,
) -> Result<Json<v0::NewRoleResponse>> {
    let data = data.into_inner();
    data.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    let server = target.as_server(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    calculate_server_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageRole);

    let config = config().await;
    if server.roles.len() >= config.features.limits.default.server_roles {
        return Err(create_error!(TooManyRoles {
            max: config.features.limits.default.server_roles,
        }));
    };

    let member_rank = query.get_member_rank();
    let rank = if let Some(given_rank) = data.rank {
        if given_rank <= member_rank.unwrap_or(i64::MIN) {
            return Err(create_error!(NotElevated));
        }

        given_rank
    } else {
        member_rank.unwrap_or(0).saturating_add(1)
    };

    let role = Role {
        name: data.name,
        rank,
        colour: None,
        hoist: false,
        permissions: Default::default(),
    };

    Ok(Json(v0::NewRoleResponse {
        id: role.create(db, &server.id).await?,
        role: role.into(),
    }))
}
