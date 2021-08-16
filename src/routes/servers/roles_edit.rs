use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result, EmptyResponse};
use crate::{database::*, notifications::events::RemoveRoleField};

use mongodb::bson::doc;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    name: Option<String>,
    #[validate(length(min = 1, max = 32))]
    colour: Option<String>,
    hoist: Option<bool>,
    rank: Option<i64>,
    remove: Option<RemoveRoleField>,
}

#[patch("/<target>/roles/<role_id>", data = "<data>")]
pub async fn req(user: User, target: Ref, role_id: String, data: Json<Data>) -> Result<EmptyResponse> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if data.name.is_none() && data.colour.is_none() && data.hoist.is_none() && data.rank.is_none() && data.remove.is_none()
    {
        return Ok(());
    }

    let target = target.fetch_server().await?;
    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_manage_roles() {
        return Err(Error::MissingPermission)
    }

    if !target.roles.contains_key(&role_id) {
        return Err(Error::InvalidRole)
    }

    let mut set = doc! {};
    let mut unset = doc! {};

    // ! FIXME: we should probably just require clients to support basic MQL incl. $set / $unset
    let mut set_update = doc! {};

    let role_key = "roles.".to_owned() + &role_id;

    if let Some(remove) = &data.remove {
        match remove {
            RemoveRoleField::Colour => {
                unset.insert(role_key.clone() + ".colour", 1);
            }
        }
    }

    if let Some(name) = &data.name {
        set.insert(role_key.clone() + ".name", name);
        set_update.insert("name", name);
    }

    if let Some(colour) = &data.colour {
        set.insert(role_key.clone() + ".colour", colour);
        set_update.insert("colour", colour);
    }

    if let Some(hoist) = &data.hoist {
        set.insert(role_key.clone() + ".hoist", hoist);
        set_update.insert("hoist", hoist);
    }

    if let Some(rank) = &data.rank {
        set.insert(role_key.clone() + ".rank", rank);
        set_update.insert("rank", rank);
    }

    let mut operations = doc! {};
    if set.len() > 0 {
        operations.insert("$set", &set);
    }

    if unset.len() > 0 {
        operations.insert("$unset", unset);
    }

    if operations.len() > 0 {
        get_collection("servers")
            .update_one(doc! { "_id": &target.id }, operations, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "server",
            })?;
    }

    ClientboundNotification::ServerRoleUpdate {
        id: target.id.clone(),
        role_id,
        data: json!(set_update),
        clear: data.remove,
    }
    .publish(target.id.clone());

    Ok(EmptyResponse {})
}
