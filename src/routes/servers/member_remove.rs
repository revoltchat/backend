use revolt_quark::{models::User, perms, Db, EmptyResponse, Permission, Ref, Result};

#[delete("/<target>/members/<member>")]
pub async fn req(db: &Db, user: User, target: Ref, member: Ref) -> Result<EmptyResponse> {
    let server = target.as_server(db).await?;
    perms(&user)
        .server(&server)
        .throw_permission(db, Permission::KickMembers)
        .await?;

    let member = member.as_member(db, &server.id).await?;
    // ! FIXME_PERMISSIONS

    member.delete(db).await.map(|_| EmptyResponse)
}
