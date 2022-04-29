use revolt_quark::{
    models::{Invite, User},
    perms, Db, EmptyResponse, Permission, Ref, Result,
};

/// # Delete Invite
///
/// Delete an invite by its id.
#[openapi(tag = "Invites")]
#[delete("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    let invite = target.as_invite(db).await?;

    if user.id == invite.creator() {
        db.delete_invite(invite.code()).await
    } else {
        match invite {
            Invite::Server { code, server, .. } => {
                let server = db.fetch_server(&server).await?;
                perms(&user)
                    .server(&server)
                    .throw_permission(db, Permission::ManageServer)
                    .await?;

                db.delete_invite(&code).await
            }
            _ => unreachable!(),
        }
    }
    .map(|_| EmptyResponse)
}
