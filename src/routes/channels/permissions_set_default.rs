use mongodb::bson::doc;
use rocket_contrib::json::Json;
use serde::{Serialize, Deserialize};

use crate::database::*;
use crate::database::permissions::channel::{ ChannelPermission, DEFAULT_PERMISSION_DM };
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};

#[derive(Serialize, Deserialize)]
pub struct Data {
    permissions: u32
}

#[put("/<target>/permissions/default", data = "<data>", rank = 1)]
pub async fn req(user: User, target: Ref, data: Json<Data>) -> Result<()> {
    let target = target.fetch_channel().await?;

    match target {
        Channel::Group { id, owner, .. } => {
            if user.id == owner {
                let permissions: u32 = ChannelPermission::View as u32 | (data.permissions & *DEFAULT_PERMISSION_DM);
                
                get_collection("channels")
                    .update_one(
                        doc! { "_id": &id },
                        doc! {
                            "$set": {
                                "permissions": permissions as i32
                            }
                        },
                        None
                    )
                    .await
                    .map_err(|_| Error::DatabaseError {
                        operation: "update_one",
                        with: "channel"
                    })?;
                
                ClientboundNotification::ChannelUpdate {
                    id: id.clone(),
                    data: json!({
                        "permissions": permissions as i32
                    }),
                    clear: None
                }
                .publish(id);
                
                Ok(())
            } else {
                Err(Error::MissingPermission)
            }
        }
        Channel::TextChannel { id, server, .. }
        | Channel::VoiceChannel { id, server, .. } => {
            let target = Ref::from_unchecked(server).fetch_server().await?;
            let perm = permissions::PermissionCalculator::new(&user)
                .with_server(&target)
                .for_server()
                .await?;
        
            if !perm.get_manage_roles() {
                return Err(Error::MissingPermission);
            }

            let permissions: u32 = ChannelPermission::View as u32 | data.permissions;
            
            get_collection("channels")
                .update_one(
                    doc! { "_id": &id },
                    doc! {
                        "$set": {
                            "default_permissions": permissions as i32
                        }
                    },
                    None
                )
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "update_one",
                    with: "channel"
                })?;
            
            ClientboundNotification::ChannelUpdate {
                id: id.clone(),
                data: json!({
                    "default_permissions": permissions as i32
                }),
                clear: None
            }
            .publish(id);

            Ok(())
        }
        _ => Err(Error::InvalidOperation)
    }
}
