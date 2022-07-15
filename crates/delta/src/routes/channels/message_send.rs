use std::collections::HashSet;

use revolt_quark::{
    models::{
        message::{Interactions, Masquerade, Reply, SendableEmbed},
        Message, User,
    },
    perms,
    web::idempotency::IdempotencyKey,
    Db, Error, Permission, Ref, Result,
};

use regex::Regex;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataMessageSend {
    /// Unique token to prevent duplicate message sending
    ///
    /// **This is deprecated and replaced by `Idempotency-Key`!**
    #[validate(length(min = 1, max = 64))]
    nonce: Option<String>,

    /// Message content to send
    #[validate(length(min = 0, max = 2000))]
    content: Option<String>,
    /// Attachments to include in message
    #[validate(length(min = 1, max = 128))]
    attachments: Option<Vec<String>>,
    /// Messages to reply to
    replies: Option<Vec<Reply>>,
    /// Embeds to include in message
    ///
    /// Text embed content contributes to the content length cap
    #[validate(length(min = 1, max = 10))]
    embeds: Option<Vec<SendableEmbed>>,
    /// Masquerade to apply to this message
    #[validate]
    masquerade: Option<Masquerade>,
    /// Information about how this message should be interacted with
    interactions: Option<Interactions>,
}

lazy_static! {
    // ignoring I L O and U is intentional
    static ref RE_MENTION: Regex = Regex::new(r"<@([0-9A-HJKMNP-TV-Z]{26})>").unwrap();
}

/// # Send Message
///
/// Sends a message to the given channel.
#[openapi(tag = "Messaging")]
#[post("/<target>/messages", data = "<data>")]
pub async fn message_send(
    db: &Db,
    user: User,
    target: Ref,
    data: Json<DataMessageSend>,
    mut idempotency: IdempotencyKey,
) -> Result<Json<Message>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    Message::validate_sum(&data.content, &data.embeds)?;

    idempotency.consume_nonce(data.nonce).await?;

    let channel = target.as_channel(db).await?;
    let mut permissions = perms(&user).channel(&channel);
    permissions
        .throw_permission_and_view_channel(db, Permission::SendMessage)
        .await?;

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
        author: user.id.clone(),
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

    // 2. Verify permissions for masquerade.
    if let Some(masq) = &message.masquerade {
        permissions
            .throw_permission(db, Permission::Masquerade)
            .await?;

        if masq.colour.is_some() {
            permissions
                .throw_permission(db, Permission::ManageRole)
                .await?;
        }
    }

    // 3. Ensure interactions information is correct
    message.interactions.validate(db).await?;

    // 4. Verify replies are valid.
    let mut replies = HashSet::new();
    if let Some(entries) = data.replies {
        if entries.len() > 5 {
            return Err(Error::TooManyReplies);
        }

        for Reply { id, mention } in entries {
            let message = Ref::from_unchecked(id).as_message(db).await?;

            replies.insert(message.id);

            if mention {
                mentions.insert(message.author);
            }
        }
    }

    if !mentions.is_empty() {
        message.mentions.replace(
            mentions
                .into_iter()
                .filter(|id| !user.has_blocked(id))
                .collect::<Vec<String>>(),
        );
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
        if !ids.is_empty() {
            permissions
                .throw_permission(db, Permission::UploadFiles)
                .await?;
        }

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

    message.create(db, &channel, Some(&user)).await?;

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
