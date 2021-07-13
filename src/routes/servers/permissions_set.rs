use mongodb::bson::doc;
use rocket_contrib::json::Json;
use serde::{Serialize, Deserialize};

use crate::database::*;
use crate::database::permissions::channel::ChannelPermission;
use crate::database::permissions::server::ServerPermission;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};

#[derive(Serialize, Deserialize)]
pub struct Values {
    server: u32,
    channel: u32
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    permissions: Values
}

#[put("/<target>/permissions/<role_id>", data = "<data>", rank = 2)]
pub async fn req(user: User, target: Ref, role_id: String, data: Json<Data>) -> Result<()> {
    let target = target.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_manage_roles() {
        return Err(Error::MissingPermission);
    }

    if !target.roles.contains_key(&role_id) {
        return Err(Error::NotFound);
    }

    let server_permissions: u32 = ServerPermission::View as u32 | data.permissions.server;
    let channel_permissions: u32 = ChannelPermission::View as u32 | data.permissions.channel;
    
    get_collection("servers")
        .update_one(
            doc! { "_id": &target.id },
            doc! {
                "$set": {
                    "roles.".to_owned() + &role_id + &".permissions": [
                        server_permissions as i32,
                        channel_permissions as i32
                    ]
                }
            },
            None
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "update_one",
            with: "server"
        })?;
    
    ClientboundNotification::ServerRoleUpdate {
        id: target.id.clone(),
        role_id,
        data: json!({
            "permissions": [
                server_permissions as i32,
                channel_permissions as i32
            ]
        }),
        clear: None
    }
    .publish(target.id);

    Ok(())
}
