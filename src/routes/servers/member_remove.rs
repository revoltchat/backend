use revolt_quark::{models::User, perms, Db, EmptyResponse, Error, Permission, Ref, Result};

#[delete("/<target>/members/<member>")]
pub async fn req(db: &Db, user: User, target: Ref, member: Ref) -> Result<EmptyResponse> {
    let server = target.as_server(db).await?;
    if !perms(&user)
        .server(&server)
        .calc(db)
        .await
        .can_kick_members()
    {
        return Error::from_permission(Permission::KickMembers);
    }

    let member = member.as_member(db, &server.id).await?;
    // ! FIXME_PERMISSIONS

    member.delete(db).await.map(|_| EmptyResponse)
}
