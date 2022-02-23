use rocket::serde::json::Json;
use serde::Deserialize;

use revolt_quark::{
    models::{channel::PartialChannel, Channel, User},
    perms, Db, Error, Override, Permission, Ref, Result,
};

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Data {
    Value { permissions: u64 },
    Field { permissions: Override },
}

#[put("/<target>/permissions/default", data = "<data>", rank = 1)]
pub async fn req(db: &Db, user: User, target: Ref, data: Json<Data>) -> Result<Json<Channel>> {
    let data = data.into_inner();

    let mut channel = target.as_channel(db).await?;
    let mut perm = perms(&user).channel(&channel);

    perm.throw_permission_and_view_channel(db, Permission::ManagePermissions)
        .await?;

    match &channel {
        Channel::Group { .. } => {
            if let Data::Value { permissions } = data {
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
            if let Data::Field { permissions } = data {
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
