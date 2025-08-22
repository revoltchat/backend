use iso8601_timestamp::Timestamp;
use revolt_database::{
    tasks,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    Database, Message, PartialMessage, User,
};
use revolt_models::v0::{self, Embed};
use revolt_permissions::{calculate_channel_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};
use validator::Validate;

/// # Edit Message
///
/// Edits a message that you've previously sent.
#[openapi(tag = "Messaging")]
#[patch("/<target>/messages/<msg>", data = "<edit>")]
pub async fn edit(
    db: &State<Database>,
    user: User,
    target: Reference<'_>,
    msg: Reference<'_>,
    edit: Json<v0::DataEditMessage>,
) -> Result<Json<v0::Message>> {
    let edit = edit.into_inner();
    edit.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    Message::validate_sum(
        &edit.content,
        edit.embeds.as_deref().unwrap_or_default(),
        user.limits().await.message_length,
    )?;

    // Ensure we have permissions to send a message
    let channel = target.as_channel(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    let permissions = calculate_channel_permissions(&mut query).await;

    permissions.throw_if_lacking_channel_permission(ChannelPermission::SendMessage)?;

    let mut message = msg.as_message_in_channel(db, channel.id()).await?;
    if message.author != user.id {
        return Err(create_error!(CannotEditMessage));
    }

    message.edited = Some(Timestamp::now_utc());
    let mut partial = PartialMessage {
        edited: message.edited,
        ..Default::default()
    };

    // 1. Handle content update
    if let Some(content) = &edit.content {
        partial.content = Some(content.clone());
    }

    // 2. Clear any auto generated embeds
    let mut new_embeds: Vec<Embed> = vec![];
    if let Some(embeds) = &message.embeds {
        for embed in embeds {
            if let Embed::Text(embed) = embed {
                new_embeds.push(Embed::Text(embed.clone()))
            }
        }
    }

    // 3. Replace if we are given new embeds
    if let Some(embeds) = edit.embeds {
        // Ensure we have permissions to send embeds
        permissions.throw_if_lacking_channel_permission(ChannelPermission::SendEmbeds)?;

        new_embeds.clear();

        for embed in embeds {
            new_embeds.push(message.create_embed(db, embed).await?);
        }
    }

    // 4. Handle attachment removal
    if let Some(remove_attachments) = &edit.remove_attachments {
        // Validate that all attachment IDs exist in the message
        if let Some(ref attachments) = message.attachments {
            for attachment_id in remove_attachments {
                if !attachments.iter().any(|file| &file.id == attachment_id) {
                    return Err(create_error!(InvalidOperation));
                }
            }

            // Remove specified attachments
            let mut updated_attachments = attachments.clone();
            let removed_files: Vec<_> = attachments
                .iter()
                .filter(|file| remove_attachments.contains(&file.id))
                .cloned()
                .collect();

            updated_attachments.retain(|file| !remove_attachments.contains(&file.id));
            partial.attachments = Some(updated_attachments);

            // Mark removed files for cleanup
            for file in removed_files {
                // TODO: Implement proper file cleanup task
                // For now we'll just log the removal
                println!("File {} removed from message {}", file.id, message.id);
            }
        } else if !remove_attachments.is_empty() {
            return Err(create_error!(InvalidOperation));
        }
    }

    if edit.remove_all_attachments == Some(true) {
        // Queue all attachments for deletion
        if let Some(ref attachments) = message.attachments {
            for file in attachments {
                // Queue file for deletion via background task
                // TODO: Implement proper file deletion queue/task
                println!(
                    "Attachment {} queued for deletion from message {}",
                    file.id, message.id
                );
            }
        }
        partial.attachments = Some(vec![]);
    }

    // 5. Handle embed suppression
    if let Some(suppress_embeds) = &edit.suppress_embeds {
        // Validate embed indices against original message embeds
        if let Some(ref original_embeds) = message.embeds {
            for &index in suppress_embeds {
                if index >= original_embeds.len() {
                    return Err(create_error!(InvalidOperation));
                }
            }
        } else if !suppress_embeds.is_empty() {
            return Err(create_error!(InvalidOperation));
        }

        // For suppression we need to work with the original embeds and filter out the suppressed ones
        if let Some(ref original_embeds) = message.embeds {
            let mut filtered_embeds = Vec::new();
            for (i, embed) in original_embeds.iter().enumerate() {
                if !suppress_embeds.contains(&i) {
                    // Keep ALL embed types that aren't being suppressed, not just Text embeds
                    filtered_embeds.push(embed.clone());
                }
            }
            new_embeds = filtered_embeds;
        }
    }

    if edit.suppress_all_embeds == Some(true) {
        new_embeds.clear();
    }

    partial.embeds = Some(new_embeds);

    message.update(db, partial, vec![]).await?;

    // Queue up a task for processing embeds if the we have sufficient permissions
    // BUT only if we're not suppressing embeds
    if permissions.has_channel_permission(ChannelPermission::SendEmbeds)
        && edit.suppress_embeds.is_none()
        && edit.suppress_all_embeds != Some(true)
    {
        if let Some(content) = edit.content {
            tasks::process_embeds::queue(
                message.channel.to_string(),
                message.id.to_string(),
                content,
            )
            .await;
        }
    }

    Ok(Json(message.into_model(None, None)))
}
