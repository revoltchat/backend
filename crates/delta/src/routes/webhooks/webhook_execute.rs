use revolt_quark::{Db, Ref, Result, Error, models::message::{Message, DataMessageSend}, web::idempotency::IdempotencyKey, types::push::MessageAuthor};
use rocket::serde::json::Json;

use validator::Validate;

/// # Executes a webhook
///
/// executes a webhook and sends a message
#[openapi(tag = "Webhooks")]
#[post("/<target>/<token>", data="<data>")]
pub async fn req(db: &Db, target: Ref, token: String, data: Json<DataMessageSend>, idempotency: IdempotencyKey) -> Result<Json<Message>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let webhook = target.as_webhook(db).await?;

    (webhook.token == token)
        .then_some(())
        .ok_or(Error::InvalidCredentials)?;

    let channel = Ref::from_unchecked(webhook.channel.clone()).as_channel(db).await?;
    let message = channel.send_message(db, data, MessageAuthor::Webhook(&webhook), idempotency).await?;

    Ok(Json(message))
}
