use revolt_quark::{Db, Ref, Result, Error, models::{webhook::{FieldsWebhook, Webhook, PartialWebhook}, File, User}, Permission, perms};
use serde::{Serialize, Deserialize};
use validator::Validate;
use rocket::serde::json::Json;

#[derive(Serialize, Deserialize, Validate, JsonSchema)]
pub struct WebhookEditBody {
    #[validate(length(min = 1, max = 32))]
    name: Option<String>,

    #[validate(length(min = 1, max = 128))]
    avatar: Option<String>,

    #[serde(default)]
    remove: Vec<FieldsWebhook>
}

/// # Edits a webhook
///
/// edits a webhook
#[openapi(tag = "Webhooks")]
#[patch("/<target>", data="<data>")]
pub async fn webhook_edit(db: &Db, target: Ref, user: User, data: Json<WebhookEditBody>) -> Result<Json<Webhook>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let mut webhook = target.as_webhook(db).await?;

    let channel = Ref::from_unchecked(webhook.channel.clone()).as_channel(db).await?;

    perms(&user)
        .channel(&channel)
        .throw_permission(db, Permission::ManageWebhooks)
        .await?;

    if data.name.is_none()
        && data.avatar.is_none()
        && data.remove.is_empty()
    {
        return Ok(Json(webhook))
    };

    let mut partial = PartialWebhook::default();

    let WebhookEditBody { name, avatar, remove } = data;

    if let Some(name) = name {
        partial.name = Some(name)
    }

    if let Some(avatar) = avatar {
        let file = File::use_avatar(db, &avatar, &webhook.id).await?;
        partial.avatar = Some(file)
    }

    webhook.update(db, partial, remove).await?;

    Ok(Json(webhook))
}
