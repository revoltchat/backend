use mongodb::bson::doc;
use rocket_contrib::json::Json;
use serde::{Serialize, Deserialize};
use validator::Contains;

use crate::database::*;
use crate::database::permissions::channel::ChannelPermission;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};

#[derive(Serialize, Deserialize)]
pub struct Data {
    permissions: u32
}

#[put("/<target>/permissions/<role>", data = "<data>", rank = 2)]
pub async fn req(user: User, target: Ref, role: String, data: Json<Data>) -> Result<()> {
    let target = target.fetch_channel().await?;

    match target {
        Channel::TextChannel { id, server, mut role_permissions, .. }
        | Channel::VoiceChannel { id, server, mut role_permissions, .. } => {
            let target = Ref::from_unchecked(server).fetch_server().await?;
            let perm = permissions::PermissionCalculator::new(&user)
                .with_server(&target)
                .for_server()
                .await?;
        
            if !perm.get_manage_roles() {
                return Err(Error::MissingPermission);
            }

            if !target.roles.has_element(&role) {
                return Err(Error::NotFound);
            }
            
            let permissions: u32 = ChannelPermission::View as u32 | data.permissions;
            
            get_collection("channels")
            .update_one(
                doc! { "_id": &id },
                doc! {
                    "$set": {
                        "role_permissions.".to_owned() + &role: permissions as i32
                    }
                },
                None
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "channel"
            })?;
            
            role_permissions.insert(role, permissions as i32);
            ClientboundNotification::ChannelUpdate {
                id: id.clone(),
                data: json!({
                    "role_permissions": role_permissions
                }),
                clear: None
            }
            .publish(id);

            Ok(())
        }
        _ => Err(Error::InvalidOperation)
    }
}
