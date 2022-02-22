use revolt_quark::{
    models::{Invite, User},
    perms, Db, EmptyResponse, Error, Permission, Ref, Result,
};

#[delete("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    let invite = target.as_invite(db).await?;

    if user.id == invite.creator() {
        db.delete_invite(invite.code()).await
    } else {
        match invite {
            Invite::Server { code, server, .. } => {
                let server = db.fetch_server(&server).await?;
                if !perms(&user)
                    .server(&server)
                    .calc(db)
                    .await
                    .can_manage_server()
                {
                    return Error::from_permission(Permission::ManageServer);
                }

                db.delete_invite(&code).await
            }
            _ => unreachable!(),
        }
    }
    .map(|_| EmptyResponse)
}
