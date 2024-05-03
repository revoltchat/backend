use revolt_database::{util::reference::Reference, Database};
use revolt_quark::{models::User, perms, Db, Error, Permission, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Deletes a webhook
///
/// Deletes a webhook
#[openapi(tag = "Webhooks")]
#[delete("/<webhook_id>")]
pub async fn webhook_delete(
    db: &State<Database>,
    legacy_db: &Db,
    user: User,
    webhook_id: Reference,
) -> Result<EmptyResponse> {
    let webhook = webhook_id.as_webhook(db).await.map_err(Error::from_core)?;
    let channel = legacy_db.fetch_channel(&webhook.channel_id).await?;

    perms(&user)
        .channel(&channel)
        .throw_permission(legacy_db, Permission::ManageWebhooks)
        .await?;

    webhook
        .delete(db)
        .await
        .map(|_| EmptyResponse)
        .map_err(Error::from_core)
}
