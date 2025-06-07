use revolt_database::{util::{permissions::DatabasePermissionQuery, reference::Reference}, Database, File, PartialRole, User};
use revolt_models::v0;
use revolt_permissions::{calculate_server_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};
use validator::Validate;

/// # Edit Role
///
/// Edit a role by its id.
#[openapi(tag = "Server Permissions")]
#[patch("/<target>/roles/<role_id>", data = "<data>")]
pub async fn edit(
    db: &State<Database>,
    user: User,
    target: Reference,
    role_id: String,
    data: Json<v0::DataEditRole>,
) -> Result<Json<v0::Role>> {
    let data = data.into_inner();
    data.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    let mut server = target.as_server(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    calculate_server_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageRole)?;

    let member_rank = query.get_member_rank().unwrap_or(i64::MIN);

    if let Some(mut role) = server.roles.remove(&role_id) {
        // Prevent us from editing roles above us
        if role.rank <= member_rank {
            return Err(create_error!(NotElevated));
        }

        let v0::DataEditRole {
            name,
            icon,
            colour,
            hoist,
            rank,
            remove,
        } = data;

        // Prevent us from moving a role above other roles
        if let Some(rank) = &rank {
            if rank <= &member_rank {
                return Err(create_error!(NotElevated));
            }
        }

        let mut partial = PartialRole {
            name,
            colour,
            hoist,
            rank,
            ..Default::default()
        };

        // 1. Remove fields from object
        if let Some(field) = &remove {
            if field.contains(&v0::FieldsRole::Icon) {
                if let Some(icon) = &role.icon {
                    db.mark_attachment_as_deleted(&icon.id).await?;
                }
            }
        }

        // 2. Apply new icon
        if let Some(icon) = icon {
            partial.icon = Some(File::use_role_icon(db, &icon, &role_id, &user.id).await?);
            role.icon = partial.icon.clone();
        }

        role.update(
            db,
            &server.id,
            &role_id,
            partial,
            remove
                .map(|v| v.into_iter().map(Into::into).collect())
                .unwrap_or_default(),
        )
        .await?;

        Ok(Json(role.into()))
    } else {
        Err(create_error!(NotFound))
    }
}
