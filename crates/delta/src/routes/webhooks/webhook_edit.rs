use revolt_database::{util::reference::Reference, Database, PartialWebhook};
use revolt_models::v0::{DataEditWebhook, Webhook};
use revolt_quark::{models::User, perms, Db, Error, Permission, Result};
use rocket::{serde::json::Json, State};
use validator::Validate;

/// # Edits a webhook
///
/// Edits a webhook
#[openapi(tag = "Webhooks")]
#[patch("/<webhook_id>", data = "<data>")]
pub async fn webhook_edit(
    db: &State<Database>,
    legacy_db: &Db,
    webhook_id: Reference,
    user: User,
    data: Json<DataEditWebhook>,
) -> Result<Json<Webhook>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let mut webhook = webhook_id.as_webhook(db).await.map_err(Error::from_core)?;
    let channel = legacy_db.fetch_channel(&webhook.channel_id).await?;

    perms(&user)
        .channel(&channel)
        .throw_permission(legacy_db, Permission::ManageWebhooks)
        .await?;

    if data.name.is_none() && data.avatar.is_none() && data.remove.is_empty() {
        return Ok(Json(webhook.into()));
    };

    let DataEditWebhook {
        name,
        avatar,
        permissions,
        remove,
    } = data;

    let mut partial = PartialWebhook {
        name,
        permissions,
        ..Default::default()
    };

    if let Some(avatar) = avatar {
        let file = db
            .find_and_use_attachment(&avatar, "avatars", "user", &webhook.id)
            .await
            .map_err(Error::from_core)?;

        partial.avatar = Some(file)
    }

    webhook
        .update(db, partial, remove.into_iter().map(|v| v.into()).collect())
        .await
        .map_err(Error::from_core)?;

    Ok(Json(webhook.into()))
}
