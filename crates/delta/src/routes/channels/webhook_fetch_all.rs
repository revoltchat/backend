use revolt_database::Database;
use revolt_models::v0::Webhook;
use revolt_quark::{models::User, perms, Db, Error, Permission, Ref, Result};
use rocket::{serde::json::Json, State};

/// # Gets all webhooks
///
/// Gets all webhooks inside the channel
#[openapi(tag = "Webhooks")]
#[get("/<channel_id>/webhooks")]
pub async fn req(
    db: &State<Database>,
    legacy_db: &Db,
    user: User,
    channel_id: Ref,
) -> Result<Json<Vec<Webhook>>> {
    let channel = channel_id.as_channel(legacy_db).await?;
    let mut permissions = perms(&user).channel(&channel);
    permissions
        .has_permission(legacy_db, Permission::ManageWebhooks)
        .await?;

    Ok(Json(
        db.fetch_webhooks_for_channel(channel.id())
            .await
            .map_err(Error::from_core)?
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<Webhook>>(),
    ))
}
