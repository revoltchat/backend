use revolt_database::util::reference::Reference;
use revolt_database::{Database, PartialWebhook};
use revolt_models::v0::{DataEditWebhook, Webhook};
use revolt_models::validator::Validate;
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};

/// # Edits a webhook
///
/// Edits a webhook with a token
#[openapi(tag = "Webhooks")]
#[patch("/<webhook_id>/<token>", data = "<data>")]
pub async fn webhook_edit_token(
    db: &State<Database>,
    webhook_id: Reference,
    token: String,
    data: Json<DataEditWebhook>,
) -> Result<Json<Webhook>> {
    let data = data.into_inner();
    data.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    let mut webhook = webhook_id.as_webhook(db).await?;
    webhook.assert_token(&token)?;

    if data.name.is_none() && data.avatar.is_none() && data.remove.is_empty() {
        return Ok(Json(webhook.into()));
    };

    let DataEditWebhook {
        name,
        avatar,
        permissions,
        remove
    } = data;

    let mut partial = PartialWebhook {
        name,
        permissions,
        ..Default::default()
    };

    if let Some(avatar) = avatar {
        let file = db
            .find_and_use_attachment(&avatar, "avatars", "user", &webhook.id)
            .await?;

        partial.avatar = Some(file)
    }

    webhook
        .update(db, partial, remove.into_iter().map(|v| v.into()).collect())
        .await?;

    Ok(Json(webhook.into()))
}
