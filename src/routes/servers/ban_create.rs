use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 1024))]
    reason: Option<String>,
}

#[put("/<server>/bans/<target>", data = "<data>")]
pub async fn req(user: User, server: Ref, target: Ref, data: Json<Data>) -> Result<()> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let server = server.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&server)
        .for_server()
        .await?;

    if !perm.get_ban_members() {
        Err(Error::MissingPermission)?
    }

    let target = target.fetch_user().await?;
    if target.id == user.id {
        return Err(Error::InvalidOperation)
    }

    if target.id == server.owner {
        return Err(Error::MissingPermission)
    }

    let mut document = doc! {
        "_id": {
            "server": &server.id,
            "user": &target.id
        }
    };

    if let Some(reason) = data.reason {
        document.insert("reason", reason);
    }

    get_collection("server_bans")
        .insert_one(document, None)
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "insert_one",
            with: "server_ban",
        })?;

    server.remove_member(&target.id, RemoveMember::Ban).await
}
