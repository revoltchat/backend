use revolt_quark::{Error, Result, models::{channel::{FieldsChannel, PartialChannel, Channel}, User, File}, Ref, Database, perms, ChannelPermission};

use mongodb::bson::doc;
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[validate(length(min = 0, max = 1024))]
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[validate(length(min = 1, max = 128))]
    icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
    #[validate(length(min = 1))]
    remove: Option<Vec<FieldsChannel>>,
}

#[patch("/<target>", data = "<data>")]
pub async fn req(db: &State<Database>, user: User, target: Ref, data: Json<Data>) -> Result<Json<Channel>> {
    let data = data.into_inner();
    data.validate().map_err(|error| Error::FailedValidation { error })?;

    let mut channel = target.as_channel(db).await?;
    if !perms(&user).channel(&channel).calc_channel(db).await.get_manage_channel() {
        return Err(Error::MissingPermission { permission: ChannelPermission::ManageChannel as i32 })
    }

    if data.name.is_none() && data.description.is_none() && data.icon.is_none() && data.nsfw.is_none() && data.remove.is_none() {
        return Ok(Json(channel))
    }

    let mut partial: PartialChannel = Default::default();
    match &mut channel {
        Channel::Group { id, name, description, icon, nsfw, .. }
        | Channel::TextChannel { id, name, description, icon, nsfw, .. }
        | Channel::VoiceChannel { id, name, description, icon, nsfw, .. } => {
            if let Some(fields) = &data.remove {
                if fields.contains(&FieldsChannel::Icon) {
                    if let Some(icon) = &icon {
                        db.mark_attachment_as_deleted(&icon.id).await?;
                    }
                }

                for field in fields {
                    match field {
                        FieldsChannel::Description => {
                            description.take();
                        },
                        FieldsChannel::Icon => {
                            icon.take();
                        },
                        _ => {}
                    }
                }
            }

            if let Some(icon_id) = data.icon {
                partial.icon = Some(File::use_icon(db, &icon_id, id).await?);
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

            db.update_channel(id, &partial, data.remove.unwrap_or_default()).await?;
        },
        _ => return Err(Error::InvalidOperation)
    };

    Ok(Json(channel))
}
