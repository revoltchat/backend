use std::collections::HashSet;

use crate::database::*;
use crate::util::idempotency::IdempotencyKey;
use crate::util::ratelimit::{Ratelimiter, RatelimitResponse};
use crate::util::result::{Error, Result};

use mongodb::bson::doc;
use regex::Regex;
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Serialize, Deserialize)]
pub struct Reply {
    id: String,
    mention: bool
}

#[derive(Validate, Serialize, Deserialize, Clone, Debug)]
pub struct SendableEmbed {
    icon_url: Option<String>,
    url: Option<String>,
    #[validate(length(min = 1, max = 100))]
    title: Option<String>,
    #[validate(length(min = 1, max = 2000))]
    description: Option<String>,
    media: Option<String>,
	colour: Option<String>,
}

impl SendableEmbed {
    pub async fn into_embed(self, message_id: String) -> Result<Embed> {
        let media = if let Some(id) = self.media {
            Some(File::find_and_use(&id, "attachments", "message", &message_id).await?)
        } else { None };

        Ok(Embed::Text(Text {
            icon_url: self.icon_url,
            url: self.url,
            title: self.title,
            description: self.description,
            media,
            colour: self.colour
        }))
    }
}

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 0, max = 2000))]
    content: String,
    #[validate(length(min = 1, max = 128))]
    attachments: Option<Vec<String>>,
    nonce: Option<String>,
    replies: Option<Vec<Reply>>,
    #[validate]
    masquerade: Option<Masquerade>,
    #[validate(length(min = 1, max = 10))]
    embeds: Option<Vec<SendableEmbed>>
}

lazy_static! {
    // ignoring I L O and U is intentional
    static ref RE_MENTION: Regex = Regex::new(r"<@([0-9A-HJKMNP-TV-Z]{26})>").unwrap();
}

#[post("/<target>/messages", data = "<message>")]
pub async fn message_send(user: User, _r: Ratelimiter, mut idempotency: IdempotencyKey, target: Ref, message: Json<Data>) -> Result<RatelimitResponse<Value>> {
    let message = message.into_inner();
    idempotency.consume_nonce(message.nonce.clone());

    message
        .validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if message.content.len() == 0
        && (message.attachments.is_none() || message.attachments.as_ref().unwrap().len() == 0)
    {
        return Err(Error::EmptyMessage);
    }

    let target = target.fetch_channel().await?;
    target.has_messaging()?;
    
    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel()
        .await?;

    if !perm.get_send_message() {
        return Err(Error::MissingPermission)
    }

    let mut mentions = HashSet::new();
    for capture in RE_MENTION.captures_iter(&message.content) {
        if let Some(mention) = capture.get(1) {
            mentions.insert(mention.as_str().to_string());
        }
    }

    if let Some(_) = &message.masquerade {
        if !perm.get_masquerade() {
            return Err(Error::MissingPermission)
        }
    }

    let mut replies = HashSet::new();
    if let Some(entries) = message.replies {
        // ! FIXME: move this to app config
        if entries.len() > 5 {
            return Err(Error::TooManyReplies)
        }

        for Reply { id, mention } in entries {
            let message = Ref::from_unchecked(id)
                .fetch_message(&target)
                .await?;
            
            replies.insert(message.id);
            
            if mention {
                mentions.insert(message.author);
            }
        }
    }

    let id = Ulid::new().to_string();
    let mut attachments = vec![];

    if let Some(ids) = &message.attachments {
        if ids.len() > 0 && !perm.get_upload_files() {
            return Err(Error::MissingPermission)
        }

        // ! FIXME: move this to app config
        if ids.len() > 5 {
            return Err(Error::TooManyAttachments)
        }

        for attachment_id in ids {
            attachments
                .push(File::find_and_use(attachment_id, "attachments", "message", &id).await?);
        }
    }

    let mut embeds = vec![];

    if let Some(sendable_embeds) = message.embeds {
        for sendable_embed in sendable_embeds {
            embeds.push(sendable_embed.into_embed(id.clone()).await?)
        }
    }

    let msg = Message {
        id,
        channel: target.id().to_string(),
        author: user.id,

        content: Content::Text(message.content.clone()),
        nonce: Some(idempotency.key),
        edited: None,
        embeds: if embeds.len() > 0 { Some(embeds) } else { None },
        attachments: if attachments.len() > 0 { Some(attachments) } else { None },
        mentions: if mentions.len() > 0 {
            Some(mentions.into_iter().collect::<Vec<String>>())
        } else {
            None
        },
        replies: if replies.len() > 0 {
            Some(replies.into_iter().collect::<Vec<String>>())
        } else {
            None
        },
        masquerade: message.masquerade
    };

    msg.clone().publish(&target, perm.get_embed_links()).await?;
    Ok(RatelimitResponse(json!(msg)))
}
