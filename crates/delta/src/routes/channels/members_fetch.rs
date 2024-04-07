use futures::future::join_all;
use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    Channel, Database, User,
};
use revolt_models::v0;
use revolt_permissions::{calculate_channel_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};

/// # Fetch Group Members
///
/// Retrieves all users who are part of this group.
#[openapi(tag = "Groups")]
#[get("/<target>/members")]
pub async fn fetch_members(
    db: &State<Database>,
    user: User,
    target: Reference,
) -> Result<Json<Vec<v0::User>>> {
    let channel = target.as_channel(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ViewChannel)?;

    if let Channel::Group { recipients, .. } = channel {
        Ok(Json(
            join_all(
                db.fetch_users(&recipients)
                    .await?
                    .into_iter()
                    .map(|other_user| other_user.into(db, &user)),
            )
            .await,
        ))
    } else {
        Err(create_error!(InvalidOperation))
    }
}
