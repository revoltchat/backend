use revolt_quark::{
    models::{server_member::MemberCompositeKey, ServerBan, User},
    perms, Db, Error, Permission, Ref, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// # Ban Information
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataBanCreate {
    /// Ban reason
    #[validate(length(min = 1, max = 1024))]
    reason: Option<String>,
}

/// # Ban User
///
/// Ban a user by their id.
#[openapi(tag = "Server Members")]
#[put("/<server>/bans/<target>", data = "<data>")]
pub async fn req(
    db: &Db,
    user: User,
    server: Ref,
    target: Ref,
    data: Json<DataBanCreate>,
) -> Result<Json<ServerBan>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let server = server.as_server(db).await?;

    if target.id == user.id {
        return Err(Error::CannotRemoveYourself);
    }

    if target.id == server.owner {
        return Err(Error::InvalidOperation);
    }

    let mut permissions = perms(&user).server(&server);

    permissions
        .throw_permission(db, Permission::BanMembers)
        .await?;

    // If member exists, check privileges against them
    if let Ok(member) = target.as_member(db, &server.id).await {
        if member.get_ranking(permissions.server.get().unwrap())
            <= permissions.get_member_rank().unwrap_or(i64::MIN)
        {
            return Err(Error::NotElevated);
        }

        server.ban_member(db, member, data.reason).await.map(Json)
    } else {
        let server_id = server.id.to_string();
        server
            .ban_user(
                db,
                MemberCompositeKey {
                    server: server_id,
                    user: target.id,
                },
                data.reason,
            )
            .await
            .map(Json)
    }
}
