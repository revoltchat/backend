use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    Channel, Database, File, PartialChannel, SystemMessage, User, AMQP,
};
use revolt_models::v0;
use revolt_permissions::{calculate_channel_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};
use validator::Validate;

/// # Edit Channel
///
/// Edit a channel object by its id.
#[openapi(tag = "Channel Information")]
#[patch("/<target>", data = "<data>")]
pub async fn edit(
    db: &State<Database>,
    amqp: &State<AMQP>,
    user: User,
    target: Reference,
    data: Json<v0::DataEditChannel>,
) -> Result<Json<v0::Channel>> {
    let data = data.into_inner();
    data.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    let mut channel = target.as_channel(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageChannel)?;

    if data.name.is_none()
        && data.description.is_none()
        && data.icon.is_none()
        && data.nsfw.is_none()
        && data.owner.is_none()
        && data.remove.is_none()
    {
        return Ok(Json(channel.into()));
    }

    let mut partial: PartialChannel = Default::default();

    // Transfer group ownership
    if let Some(new_owner) = data.owner {
        if let Channel::Group {
            owner, recipients, ..
        } = &mut channel
        {
            // Make sure we are the owner of this group
            if owner != &user.id {
                return Err(create_error!(NotOwner));
            }

            // Ensure user is part of group
            if !recipients.contains(&new_owner) {
                return Err(create_error!(NotInGroup));
            }

            // Transfer ownership
            partial.owner = Some(new_owner.to_string());
            let old_owner = std::mem::replace(owner, new_owner.to_string());

            // Notify clients
            SystemMessage::ChannelOwnershipChanged {
                from: old_owner,
                to: new_owner,
            }
        } else {
            return Err(create_error!(InvalidOperation));
        }
        .into_message(channel.id().to_string())
        .send(
            db,
            Some(amqp),
            user.as_author_for_system(),
            None,
            None,
            &channel,
            false,
        )
        .await
        .ok();
    }

    match &mut channel {
        Channel::Group {
            id,
            name,
            description,
            icon,
            nsfw,
            ..
        }
        | Channel::TextChannel {
            id,
            name,
            description,
            icon,
            nsfw,
            ..
        }
        | Channel::VoiceChannel {
            id,
            name,
            description,
            icon,
            nsfw,
            ..
        } => {
            if let Some(fields) = &data.remove {
                if fields.contains(&v0::FieldsChannel::Icon) {
                    if let Some(icon) = &icon {
                        db.mark_attachment_as_deleted(&icon.id).await?;
                    }
                }

                for field in fields {
                    match field {
                        v0::FieldsChannel::Description => {
                            description.take();
                        }
                        v0::FieldsChannel::Icon => {
                            icon.take();
                        }
                        _ => {}
                    }
                }
            }

            if let Some(icon_id) = data.icon {
                partial.icon = Some(File::use_channel_icon(db, &icon_id, id, &user.id).await?);
                *icon = partial.icon.clone();
            }

            if let Some(new_name) = data.name {
                *name = new_name.clone();
                partial.name = Some(new_name);
            }

            if let Some(new_description) = data.description {
                partial.description = Some(new_description);
                *description = partial.description.clone();
            }

            if let Some(new_nsfw) = data.nsfw {
                *nsfw = new_nsfw;
                partial.nsfw = Some(new_nsfw);
            }

            // Send out mutation system messages.
            if let Channel::Group { .. } = &channel {
                if let Some(name) = &partial.name {
                    SystemMessage::ChannelRenamed {
                        name: name.to_string(),
                        by: user.id.clone(),
                    }
                    .into_message(channel.id().to_string())
                    .send(
                        db,
                        Some(amqp),
                        user.as_author_for_system(),
                        None,
                        None,
                        &channel,
                        false,
                    )
                    .await
                    .ok();
                }

                if partial.description.is_some() {
                    SystemMessage::ChannelDescriptionChanged {
                        by: user.id.clone(),
                    }
                    .into_message(channel.id().to_string())
                    .send(
                        db,
                        Some(amqp),
                        user.as_author_for_system(),
                        None,
                        None,
                        &channel,
                        false,
                    )
                    .await
                    .ok();
                }

                if partial.icon.is_some() {
                    SystemMessage::ChannelIconChanged {
                        by: user.id.clone(),
                    }
                    .into_message(channel.id().to_string())
                    .send(
                        db,
                        Some(amqp),
                        user.as_author_for_system(),
                        None,
                        None,
                        &channel,
                        false,
                    )
                    .await
                    .ok();
                }
            }

            channel
                .update(
                    db,
                    partial,
                    data.remove
                        .unwrap_or_default()
                        .into_iter()
                        .map(|f| f.into())
                        .collect(),
                )
                .await?;
        }
        _ => return Err(create_error!(InvalidOperation)),
    };

    Ok(Json(channel.into()))
}
