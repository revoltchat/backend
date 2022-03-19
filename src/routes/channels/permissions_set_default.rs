use rocket::serde::json::Json;
use serde::Deserialize;

use revolt_quark::{
    models::{channel::PartialChannel, Channel, User},
    perms, Db, Error, Override, Permission, Ref, Result,
};

/// # Permission Value
#[derive(Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum DataDefaultChannelPermissions {
    Value {
        /// Permission values to set for members in a `Group`
        permissions: u64,
    },
    Field {
        /// Allow / deny values to set for members in this `TextChannel` or `VoiceChannel`
        permissions: Override,
    },
}

/// # Set Default Permission
///
/// Sets permissions for the default role in this channel.
///
/// Channel must be a `Group`, `TextChannel` or `VoiceChannel`.
#[openapi(tag = "Channel Permissions")]
#[put("/<target>/permissions/default", data = "<data>", rank = 1)]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    data: Json<DataDefaultChannelPermissions>,
) -> Result<Json<Channel>> {
    let data = data.into_inner();

    let mut channel = target.as_channel(db).await?;
    let mut perm = perms(&user).channel(&channel);

    perm.throw_permission_and_view_channel(db, Permission::ManagePermissions)
        .await?;

    match &channel {
        Channel::Group { .. } => {
            if let DataDefaultChannelPermissions::Value { permissions } = data {
                channel
                    .update(
                        db,
                        PartialChannel {
                            permissions: Some(permissions as i64),
                            ..Default::default()
                        },
                        vec![],
                    )
                    .await?;
            } else {
                return Err(Error::InvalidOperation);
            }
        }
        Channel::TextChannel {
            default_permissions,
            ..
        }
        | Channel::VoiceChannel {
            default_permissions,
            ..
        } => {
            if let DataDefaultChannelPermissions::Field { permissions } = data {
                perm.throw_permission_override(
                    db,
                    default_permissions.map(|x| x.into()),
                    permissions,
                )
                .await?;

                channel
                    .update(
                        db,
                        PartialChannel {
                            default_permissions: Some(permissions.into()),
                            ..Default::default()
                        },
                        vec![],
                    )
                    .await?;
            } else {
                return Err(Error::InvalidOperation);
            }
        }
        _ => return Err(Error::InvalidOperation),
    }

    Ok(Json(channel))
}
