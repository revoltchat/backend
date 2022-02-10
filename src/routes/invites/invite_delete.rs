use revolt_quark::{
    models::{Invite, User},
    perms, Db, EmptyResponse, Error, Ref, Result, ServerPermission,
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
                    .calc_server(db)
                    .await
                    .get_manage_server()
                {
                    return Err(Error::MissingPermission {
                        permission: ServerPermission::ManageServer as i32,
                    });
                }

                db.delete_invite(&code).await
            }
            _ => unreachable!(),
        }
    }
    .map(|_| EmptyResponse)
}
