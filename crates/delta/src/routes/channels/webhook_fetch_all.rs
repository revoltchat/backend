use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    Database, User,
};
use revolt_models::v0::Webhook;
use revolt_permissions::{calculate_channel_permissions, ChannelPermission};
use revolt_result::Result;
use rocket::{serde::json::Json, State};

/// # Gets all webhooks
///
/// Gets all webhooks inside the channel
#[openapi(tag = "Webhooks")]
#[get("/<channel_id>/webhooks")]
pub async fn req(
    db: &State<Database>,
    user: User,
    channel_id: Reference,
) -> Result<Json<Vec<Webhook>>> {
    let channel = channel_id.as_channel(db).await?;

    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ViewChannel)?;

    Ok(Json(
        db.fetch_webhooks_for_channel(&channel.id())
            .await?
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<Webhook>>(),
    ))
}
