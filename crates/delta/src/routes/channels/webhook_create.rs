use revolt_quark::{models::{User, Webhook, File}, perms, Db, Error, Permission, Ref, Result};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct CreateWebhookBody {
    #[validate(length(min = 1, max = 32))]
    name: String,

    #[validate(length(min = 1, max = 128))]
    avatar: Option<String>
}

/// # Creates a webhook
///
/// creates a webhook which 3rd party platforms can use to send messages
#[openapi(tag = "Webhooks")]
#[post("/<target>/webhooks", data = "<data>")]
pub async fn req(db: &Db, user: User, target: Ref, data: Json<CreateWebhookBody>) -> Result<Json<Webhook>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let channel = target.as_channel(db).await?;
    let mut permissions = perms(&user).channel(&channel);
    permissions
        .has_permission(db, Permission::ManageWebhooks)
        .await?;

    let webhook_id = Ulid::new().to_string();

    let avatar = match &data.avatar {
        Some(id) => Some(File::use_avatar(db, id, &webhook_id).await?),
        None => None
    };

    let webhook = Webhook {
        id: webhook_id,
        name: data.name,
        avatar,
        channel: channel.id().to_string(),
        token: nanoid::nanoid!(64)
    };

    webhook.create(db).await?;

    Ok(Json(webhook))
}
