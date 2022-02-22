use std::collections::HashSet;

use revolt_quark::{
    models::{
        message::{Content, Masquerade, Reply, SendableEmbed},
        Message, User,
    },
    perms, Db, Error, Permission, Ref, Result,
};

use regex::Regex;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

use crate::util::idempotency::IdempotencyKey;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    nonce: Option<String>,

    #[validate(length(min = 0, max = 2000))]
    content: String,
    #[validate(length(min = 1, max = 128))]
    attachments: Option<Vec<String>>,
    replies: Option<Vec<Reply>>,
    #[validate]
    masquerade: Option<Masquerade>,
    #[validate(length(min = 1, max = 10))]
    embeds: Option<Vec<SendableEmbed>>,
}

lazy_static! {
    // ignoring I L O and U is intentional
    static ref RE_MENTION: Regex = Regex::new(r"<@([0-9A-HJKMNP-TV-Z]{26})>").unwrap();
}

#[post("/<target>/messages", data = "<data>")]
pub async fn message_send(
    db: &Db,
    user: User,
    target: Ref,
    data: Json<Data>,
    mut idempotency: IdempotencyKey,
) -> Result<Json<Message>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    idempotency.consume_nonce(data.nonce).await?;

    let channel = target.as_channel(db).await?;
    let permissions = perms(&user).channel(&channel).calc(db).await;

    if !permissions.can_send_message() {
        return Error::from_permission(Permission::SendMessage);
    }

    if data.content.is_empty()
        && (data.attachments.is_none() || data.attachments.as_ref().unwrap().is_empty())
    {
        return Err(Error::EmptyMessage);
    }

    let message_id = Ulid::new().to_string();
    let mut message = Message {
        id: message_id.clone(),
        channel: channel.as_id(),
        author: user.id,
        ..Default::default()
    };

    // 1. Parse mentions in message.
    let mut mentions = HashSet::new();
    for capture in RE_MENTION.captures_iter(&data.content) {
        if let Some(mention) = capture.get(1) {
            mentions.insert(mention.as_str().to_string());
        }
    }

    // 2. Verify permissions for masquerade.
    if data.masquerade.is_some() && !permissions.can_masquerade() {
        return Error::from_permission(Permission::Masquerade);
    }

    // 3. Verify replies are valid.
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
        message
            .mentions
            .replace(mentions.into_iter().collect::<Vec<String>>());
    }

    if !replies.is_empty() {
        message
            .replies
            .replace(replies.into_iter().collect::<Vec<String>>());
    }

    // 4. Add attachments to message.
    let mut attachments = vec![];
    if let Some(ids) = &data.attachments {
        if !ids.is_empty() && !permissions.can_upload_files() {
            return Error::from_permission(Permission::UploadFiles);
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

    // 6. Set content
    message.content = Content::Text(data.content);

    // 7. Pass-through nonce value for clients
    message.nonce = Some(idempotency.into_key());

    message.create(db).await?;
    Ok(Json(message))
}
