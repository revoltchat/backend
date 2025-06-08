use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    Database, User,
};
use revolt_models::v0;
use revolt_permissions::{calculate_server_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};

/// # Edits server roles ranks
///
/// Edit's server role's ranks.
#[openapi(tag = "Server Permissions")]
#[patch("/<target>/roles/ranks", data = "<data>")]
pub async fn edit_role_ranks(
    db: &State<Database>,
    user: User,
    target: Reference,
    data: Json<v0::DataEditRoleRanks>,
) -> Result<Json<v0::Server>> {
    let data = data.into_inner();

    let mut server = target.as_server(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    calculate_server_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageRole)?;

    let existing_order = server.roles.keys().cloned().collect::<Vec<_>>();
    let new_order = data.ranks.clone().into_iter().collect::<Vec<_>>();

    // Verify all roles are in the new ordering
    if data.ranks.len() != server.roles.len()
        && server.roles.iter().all(|(id, _)| data.ranks.contains(id))
    {
        return Err(create_error!(InvalidOperation));
    }

    // Don't have to check what the user cant modify if they are the server owner
    if server.owner != user.id {
        let member_top_rank = query.get_member_rank();

        if server
            .roles
            .iter()
            // Find all roles above the member which we should not be able to reorder
            .filter(|(_, role)| {
                if let Some(top_rank) = member_top_rank {
                    role.rank <= top_rank
                } else {
                    true
                }
            })
            // Check if user is trying to reorder roles they can't reorder (as found previously)
            .any(|(id, _)| {
                existing_order
                    .iter()
                    .position(|existing_id| id == existing_id)
                    != new_order.iter().position(|new_id| id == new_id)
            })
        {
            return Err(create_error!(NotElevated));
        }
    }

    server.set_role_ordering(db, new_order).await?;

    Ok(Json(server.into()))
}
