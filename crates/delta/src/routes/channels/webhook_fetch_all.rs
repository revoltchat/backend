use revolt_quark::{models::{User, Webhook}, perms, Db, Permission, Ref, Result};
use rocket::serde::json::Json;

/// # Gets all webhooks
///
/// gets all webhooks inside the channel
#[openapi(tag = "Webhooks")]
#[get("/<target>/webhooks")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Vec<Webhook>>> {
    let channel = target.as_channel(db).await?;
    let mut permissions = perms(&user).channel(&channel);
    permissions
        .has_permission(db, Permission::ManageWebhooks)
        .await?;

    let webhooks = db.fetch_webhooks_for_channel(channel.id()).await?;

    Ok(Json(webhooks))
}
