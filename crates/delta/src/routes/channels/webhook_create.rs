use revolt_database::{Database, Webhook};
use revolt_quark::{
    models::{Channel, User},
    perms, Db, Error, Permission, Ref, Result,
    DEFAULT_WEBHOOK_PERMISSIONS,
};
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct CreateWebhookBody {
    #[validate(length(min = 1, max = 32))]
    name: String,

    #[validate(length(min = 1, max = 128))]
    avatar: Option<String>,
}

/// # Creates a webhook
///
/// Creates a webhook which 3rd party platforms can use to send messages
#[openapi(tag = "Webhooks")]
#[post("/<target>/webhooks", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    legacy_db: &Db,
    user: User,
    target: Ref,
    data: Json<CreateWebhookBody>,
) -> Result<Json<revolt_models::v0::Webhook>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let channel = target.as_channel(legacy_db).await?;

    if !matches!(channel, Channel::TextChannel { .. } | Channel::Group { .. }) {
        return Err(Error::InvalidOperation);
    }

    let mut permissions = perms(&user).channel(&channel);
    permissions
        .has_permission(legacy_db, Permission::ManageWebhooks)
        .await?;

    let webhook_id = Ulid::new().to_string();

    let avatar = match &data.avatar {
        Some(id) => Some(
            db.find_and_use_attachment(id, "avatars", "user", &webhook_id)
                .await
                .map_err(Error::from_core)?,
        ),
        None => None,
    };

    let webhook = Webhook {
        id: webhook_id,
        name: data.name,
        avatar,
        channel_id: channel.id().to_string(),
        permissions: *DEFAULT_WEBHOOK_PERMISSIONS,
        token: Some(nanoid::nanoid!(64)),
    };

    webhook.create(db).await.map_err(Error::from_core)?;

    Ok(Json(webhook.into()))
}
