use revolt_database::{util::reference::Reference, Database};
use revolt_quark::{
    models::message::{DataMessageSend, Message},
    types::push::MessageAuthor,
    web::idempotency::IdempotencyKey,
    Db, Error, Result,
};
use rocket::{serde::json::Json, State};

use validator::Validate;

/// # Executes a webhook
///
/// Executes a webhook and sends a message
#[openapi(tag = "Webhooks")]
#[post("/<webhook_id>/<token>", data = "<data>")]
pub async fn webhook_execute(
    db: &State<Database>,
    legacy_db: &Db,
    webhook_id: Reference,
    token: String,
    data: Json<DataMessageSend>,
    idempotency: IdempotencyKey,
) -> Result<Json<Message>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let webhook = webhook_id.as_webhook(db).await.map_err(Error::from_core)?;
    webhook.assert_token(&token).map_err(Error::from_core)?;

    data.validate_webhook_permissions(webhook.permissions)?;

    let channel = legacy_db.fetch_channel(&webhook.channel_id).await?;
    let message = channel
        .send_message(
            legacy_db,
            data,
            MessageAuthor::Webhook(&webhook.into()),
            idempotency,
            true,
        )
        .await?;

    Ok(Json(message))
}
