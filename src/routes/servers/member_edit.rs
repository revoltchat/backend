use revolt_quark::{
    models::{
        server_member::{FieldsMember, PartialMember},
        File, Member, User,
    },
    perms, Db, Error, Ref, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// # Member Data
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataMemberEdit {
    /// Member nickname
    #[validate(length(min = 1, max = 32))]
    nickname: Option<String>,
    /// Attachment Id to set for avatar
    avatar: Option<String>,
    /// Array of role ids
    roles: Option<Vec<String>>,
    /// Fields to remove from channel object
    #[validate(length(min = 1))]
    remove: Option<Vec<FieldsMember>>,
}

/// # Edit Member
///
/// Edit a member by their id.
#[openapi(tag = "Server Members")]
#[patch("/<server>/members/<target>", data = "<data>")]
pub async fn req(
    db: &Db,
    user: User,
    server: Ref,
    target: Ref,
    data: Json<DataMemberEdit>,
) -> Result<Json<Member>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let server = server.as_server(db).await?;
    perms(&user).server(&server).calc(db).await?;

    let mut member = target.as_member(db, &server.id).await?;

    let DataMemberEdit {
        nickname,
        avatar,
        roles,
        remove,
    } = data;

    let mut partial = PartialMember {
        nickname,
        roles,
        ..Default::default()
    };

    // ! FIXME: calculate permission against member
    // ! FIXME: also check roles exist lol

    // 1. Remove fields from object
    if let Some(fields) = &remove {
        if fields.contains(&FieldsMember::Avatar) {
            if let Some(avatar) = &member.avatar {
                db.mark_attachment_as_deleted(&avatar.id).await?;
            }
        }
    }

    // 2. Apply new avatar
    if let Some(avatar) = avatar {
        partial.avatar = Some(File::use_avatar(db, &avatar, &user.id).await?);
    }

    member
        .update(db, partial, remove.unwrap_or_default())
        .await?;

    Ok(Json(member))
}
