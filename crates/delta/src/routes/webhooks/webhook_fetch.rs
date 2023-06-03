use revolt_database::Database;
use revolt_models::v0::{ResponseWebhook, Webhook};
use revolt_quark::{models::User, perms, Db, Error, Permission, Result};
use rocket::{serde::json::Json, State};

/// # Gets a webhook
///
/// Gets a webhook
#[openapi(tag = "Webhooks")]
#[get("/<webhook_id>")]
pub async fn webhook_fetch(
    db: &State<Database>,
    legacy_db: &Db,
    webhook_id: String,
    user: User,
) -> Result<Json<ResponseWebhook>> {
    let webhook = db
        .fetch_webhook(&webhook_id)
        .await
        .map_err(Error::from_core)?;

    let channel = legacy_db.fetch_channel(&webhook.channel_id).await?;

    perms(&user)
        .channel(&channel)
        .throw_permission(legacy_db, Permission::ViewChannel)
        .await?;

    Ok(Json(std::convert::Into::<Webhook>::into(webhook).into()))
}
