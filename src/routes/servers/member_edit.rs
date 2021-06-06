use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};
use crate::{database::*, notifications::events::RemoveMemberField};

use mongodb::bson::{doc, to_document};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    nickname: Option<String>,
    avatar: Option<String>,
    remove: Option<RemoveMemberField>,
}

#[patch("/<server>/members/<target>", data = "<data>")]
pub async fn req(user: User, server: Ref, target: String, data: Json<Data>) -> Result<()> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if data.nickname.is_none() && data.avatar.is_none() && data.remove.is_none()
    {
        return Ok(());
    }

    let server = server.fetch_server().await?;
    let target = Ref::from(target)?.fetch_member(&server.id).await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&server)
        .for_server()
        .await?;

    if target.id.user == user.id {
        if (data.nickname.is_some() && !perm.get_change_nickname()) ||
            (data.avatar.is_some() && !perm.get_change_avatar()) {
            return Err(Error::MissingPermission)
        }

        if let Some(remove) = &data.remove {
            if match remove {
                RemoveMemberField::Avatar => !perm.get_change_avatar(),
                RemoveMemberField::Nickname => !perm.get_change_nickname()
            } {
                return Err(Error::MissingPermission)
            }
        }
    } else {
        if data.avatar.is_some() || (data.nickname.is_some() && !perm.get_manage_nicknames()) {
            return Err(Error::MissingPermission)
        }

        if let Some(remove) = &data.remove {
            if match remove {
                RemoveMemberField::Avatar => !perm.get_remove_avatars(),
                RemoveMemberField::Nickname => !perm.get_manage_nicknames()
            } {
                return Err(Error::MissingPermission)
            }
        }
    }

    let mut set = doc! {};
    let mut unset = doc! {};

    let mut remove_avatar = false;
    if let Some(remove) = &data.remove {
        match remove {
            RemoveMemberField::Avatar => {
                unset.insert("avatar", 1);
                remove_avatar = true;
            }
            RemoveMemberField::Nickname => {
                unset.insert("nickname", 1);
            }
        }
    }

    if let Some(name) = &data.nickname {
        set.insert("nickname", name);
    }

    if let Some(attachment_id) = &data.avatar {
        let attachment = File::find_and_use(&attachment_id, "avatars", "user", &target.id.user).await?;
        set.insert(
            "avatar",
            to_document(&attachment).map_err(|_| Error::DatabaseError {
                operation: "to_document",
                with: "attachment",
            })?,
        );

        remove_avatar = true;
    }

    let mut operations = doc! {};
    if set.len() > 0 {
        operations.insert("$set", &set);
    }

    if unset.len() > 0 {
        operations.insert("$unset", unset);
    }

    if operations.len() > 0 {
        get_collection("server_members")
            .update_one(doc! { "_id.server": &server.id, "_id.user": &target.id.user }, operations, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "server_member",
            })?;
    }

    ClientboundNotification::ServerMemberUpdate {
        id: target.id.clone(),
        data: json!(set),
        clear: data.remove,
    }
    .publish(server.id.clone());

    let Member { avatar, .. } = target;

    if remove_avatar {
        if let Some(old_avatar) = avatar {
            old_avatar.delete().await?;
        }
    }

    Ok(())
}
