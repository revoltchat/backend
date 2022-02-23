use revolt_quark::{models::User, perms, Db, EmptyResponse, Error, Permission, Ref, Result};

#[delete("/<target>/members/<member>")]
pub async fn req(db: &Db, user: User, target: Ref, member: Ref) -> Result<EmptyResponse> {
    let server = target.as_server(db).await?;

    if member.id == user.id {
        return Err(Error::CannotRemoveYourself);
    }

    if member.id == server.owner {
        return Err(Error::InvalidOperation);
    }

    let mut permissions = perms(&user).server(&server);

    permissions
        .throw_permission(db, Permission::KickMembers)
        .await?;

    let member = member.as_member(db, &server.id).await?;

    if member.get_ranking(permissions.server.get().unwrap())
        <= permissions.get_member_rank().unwrap_or(i64::MIN)
    {
        return Err(Error::NotElevated);
    }

    member.delete(db).await.map(|_| EmptyResponse)
}
