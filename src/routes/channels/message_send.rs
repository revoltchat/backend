use std::collections::HashSet;

use crate::database::*;
use crate::util::ratelimit::{Ratelimiter, RatelimitResponse};
use crate::util::result::{Error, Result};

use mongodb::{bson::doc, options::FindOneOptions};
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

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 0, max = 2000))]
    content: String,
    // Maximum length of 36 allows both ULIDs and UUIDs.
    #[validate(length(min = 1, max = 36))]
    nonce: String,
    #[validate(length(min = 1, max = 128))]
    attachments: Option<Vec<String>>,
    replies: Option<Vec<Reply>>,
}

lazy_static! {
    static ref RE_ULID: Regex = Regex::new(r"<@([0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26})>").unwrap();
}

#[post("/<target>/messages", data = "<message>")]
pub async fn message_send(_r: Ratelimiter, user: User, target: Ref, message: Json<Data>) -> Result<RatelimitResponse<Value>> {
    let message = message.into_inner();
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

    if get_collection("messages")
        .find_one(
            doc! {
                "nonce": &message.nonce
            },
            FindOneOptions::builder()
                .projection(doc! { "_id": 1 })
                .build(),
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find_one",
            with: "message",
        })?
        .is_some()
    {
        Err(Error::DuplicateNonce)?
    }

    let id = Ulid::new().to_string();

    let mut mentions = HashSet::new();
    if let Some(captures) = RE_ULID.captures_iter(&message.content).next() {
        // ! FIXME: in the future, verify in group so we can send out push
        mentions.insert(captures[1].to_string());
    }

    let mut replies = HashSet::new();
    if let Some(entries) = message.replies {
        // ! FIXME: move this to app config
        if entries.len() >= 5 {
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

    let mut attachments = vec![];
    if let Some(ids) = &message.attachments {
        if ids.len() > 0 && !perm.get_upload_files() {
            return Err(Error::MissingPermission)
        }

        // ! FIXME: move this to app config
        if ids.len() >= 5 {
            return Err(Error::TooManyAttachments)
        }

        for attachment_id in ids {
            attachments
                .push(File::find_and_use(attachment_id, "attachments", "message", &id).await?);
        }
    }

    let msg = Message {
        id,
        channel: target.id().to_string(),
        author: user.id,

        content: Content::Text(message.content.clone()),
        nonce: Some(message.nonce.clone()),
        edited: None,
        embeds: None,

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
    };

    msg.clone().publish(&target, perm.get_embed_links()).await?;

    Ok(RatelimitResponse(json!(msg)))
}
