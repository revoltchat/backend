use revolt_quark::{models::User, perms, Db, EmptyResponse, Error, Permission, Ref, Result};

#[delete("/<server>/bans/<target>")]
pub async fn req(db: &Db, user: User, server: Ref, target: Ref) -> Result<EmptyResponse> {
    let server = server.as_server(db).await?;
    if !perms(&user)
        .server(&server)
        .calc(db)
        .await
        .can_ban_members()
    {
        return Error::from_permission(Permission::BanMembers);
    }

    let ban = target.as_ban(db, &server.id).await?;
    db.delete_ban(&ban.id).await.map(|_| EmptyResponse)
}
