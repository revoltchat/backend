use revolt_database::{events::client::EventV1, util::{permissions::DatabasePermissionQuery, reference::Reference}, Database, PartialRole, User};
use revolt_permissions::{calculate_server_permissions, ChannelPermission};
use rocket::{serde::json::Json, State};
use revolt_result::{create_error, Result};
use revolt_models::v0;

/// # Edits server roles ranks
///
/// Edit's server role's ranks.
#[openapi(tag = "Server Permissions")]
#[patch("/<target>/roles/ranks", data = "<data>", rank = 1)]
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

    // verify all roles are in the new ordering
    if data.ranks.len() != server.roles.len() && server.roles.iter().all(|(id, _)| data.ranks.contains(id)) {
        return Err(create_error!(InvalidOperation))
    }

    let member_top_rank = query.get_member_rank();

    // find all roles above the member which we should not be able to reorder
    let cant_modify = server.roles.clone()
        .into_iter()
        .filter(|(_, role)| if let Some(top_rank) = member_top_rank { role.rank <= top_rank } else { true })
        .collect::<Vec<_>>();

    // check if any roles which we cant reorder have tried to been reordered
    if cant_modify.iter()
        .any(|(id, _)| {
            let existing_rank = existing_order.iter().position(|existing_id| id == existing_id);
            let new_rank = new_order.iter().position(|new_id| id == new_id);

            existing_rank != new_rank
        })
    {
        return Err(create_error!(NotElevated))
    }

    // update the roles which have had their rank changed
    for (role_id, role) in &mut server.roles {
        let new_rank = new_order.iter().position(|id| id == role_id).unwrap() as i64;  // unwrap - we check if its in the data earlier on

        if new_rank != role.rank {
            // cant use Role::update here because we dont want to publish seperate role update events
            let partial = PartialRole {
                rank: Some(new_rank),
                ..Default::default()
            };

            db.update_role(&server.id, role_id, &partial, Vec::new()).await?;

            role.apply_options(PartialRole {
                rank: Some(new_rank),
                ..Default::default()
            });
        }
    }

    // publish bulk update event
    EventV1::ServerRoleRanksUpdate {
        ranks: new_order
    }.p(server.id.clone()).await;

    Ok(Json(server.into()))
}
