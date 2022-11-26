use std::collections::HashSet;

use revolt_quark::{Db, Ref, Result, Error, models::{Message, message::Reply}, web::idempotency::IdempotencyKey, types::push::MessageAuthor};
use rocket::serde::json::Json;
use ulid::Ulid;
use crate::routes::channels::message_send::{DataMessageSend, RE_MENTION};
use validator::Validate;

/// # Executes a webhook
///
/// executes a webhook and sends a message
#[openapi(tag = "Webhooks")]
#[post("/<target>/<token>", data="<data>")]
pub async fn req(db: &Db, target: Ref, token: String, data: Json<DataMessageSend>, mut idempotency: IdempotencyKey) -> Result<Json<Message>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let webhook = target.as_webhook(db).await?;

    (webhook.token == token)
        .then_some(())
        .ok_or(Error::InvalidCredentials)?;

    Message::validate_sum(&data.content, &data.embeds)?;

    idempotency.consume_nonce(data.nonce).await?;

    let channel = Ref::from_unchecked(webhook.channel.clone()).as_channel(db).await?;

    if (data.content.as_ref().map_or(true, |v| v.is_empty()))
        && (data.attachments.as_ref().map_or(true, |v| v.is_empty()))
        && (data.embeds.as_ref().map_or(true, |v| v.is_empty()))
    {
        return Err(Error::EmptyMessage);
    }

    let message_id = Ulid::new().to_string();
    let mut message = Message {
        id: message_id.clone(),
        channel: channel.id().to_string(),
        webhook: Some(webhook.id.clone()),
        masquerade: data.masquerade,
        interactions: data.interactions.unwrap_or_default(),
        ..Default::default()
    };

    // 1. Parse mentions in message.
    let mut mentions = HashSet::new();
    if let Some(content) = &data.content {
        for capture in RE_MENTION.captures_iter(content) {
            if let Some(mention) = capture.get(1) {
                mentions.insert(mention.as_str().to_string());
            }
        }
    }

    // 4. Verify replies are valid.
    let mut replies = HashSet::new();
    if let Some(entries) = data.replies {
        if entries.len() > 5 {
            return Err(Error::TooManyReplies);
        }

        for Reply { id, mention } in entries {
            let message = Ref::from_unchecked(id).as_message(db).await?;

            if mention {
                mentions.insert(message.author_id().to_owned());
            }

            replies.insert(message.id);
        }
    }

    if !mentions.is_empty() {
        message.mentions.replace(mentions.into_iter().collect());
    }

    if !replies.is_empty() {
        message
            .replies
            .replace(replies.into_iter().collect::<Vec<String>>());
    }

    // 5. Process included embeds.
    let mut embeds = vec![];
    if let Some(sendable_embeds) = data.embeds {
        for sendable_embed in sendable_embeds {
            embeds.push(sendable_embed.into_embed(db, message_id.clone()).await?)
        }
    }

    if !embeds.is_empty() {
        message.embeds.replace(embeds);
    }

    // 6. Add attachments to message.
    let mut attachments = vec![];
    if let Some(ids) = &data.attachments {
        // ! FIXME: move this to app config
        if ids.len() > 5 {
            return Err(Error::TooManyAttachments);
        }

        for attachment_id in ids {
            attachments.push(
                db.find_and_use_attachment(attachment_id, "attachments", "message", &message_id)
                    .await?,
            );
        }
    }

    if !attachments.is_empty() {
        message.attachments.replace(attachments);
    }

    // 7. Set content
    message.content = data.content;

    // 8. Pass-through nonce value for clients
    message.nonce = Some(idempotency.into_key());

    message.create(db, &channel, Some(MessageAuthor::Webhook(&webhook))).await?;

    // Queue up a task for processing embeds
    if let Some(content) = &message.content {
        revolt_quark::tasks::process_embeds::queue(
            channel.id().to_string(),
            message.id.to_string(),
            content.clone(),
        )
        .await;
    }

    Ok(Json(message))

}
