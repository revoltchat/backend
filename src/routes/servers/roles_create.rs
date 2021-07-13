use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};

use ulid::Ulid;
use mongodb::bson::doc;
use validator::Validate;
use serde::{Serialize, Deserialize};
use rocket_contrib::json::{Json, JsonValue};

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    name: String
}

#[post("/<target>/roles", data = "<data>")]
pub async fn req(user: User, target: Ref, data: Json<Data>) -> Result<JsonValue> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;
    
    let target = target.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_manage_roles() {
        Err(Error::MissingPermission)?
    }

    let id = Ulid::new().to_string();
    let perm_tuple = (
        *permissions::server::DEFAULT_PERMISSION as i32,
        *permissions::channel::DEFAULT_PERMISSION_SERVER as i32
    );

    get_collection("servers")
        .update_one(
            doc! {
                "_id": &target.id
            },
            doc! {
                "$set": {
                    "roles.".to_owned() + &id: {
                        "name": &data.name,
                        "permissions": [
                            &perm_tuple.0,
                            &perm_tuple.1
                        ]
                    } 
                }
            },
            None
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "update_one",
            with: "servers"
        })?;
    
    ClientboundNotification::ServerRoleUpdate {
        id: target.id.clone(),
        role_id: id.clone(),
        data: json!({
            "name": data.name,
            "permissions": &perm_tuple
        }),
        clear: None
    }
    .publish(target.id);

    Ok(json!({ "id": id, "permissions": perm_tuple }))
}
