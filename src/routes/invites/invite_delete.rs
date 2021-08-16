use crate::database::*;
use crate::util::result::{Error, Result, EmptyResponse};

#[delete("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<EmptyResponse> {
    let target = target.fetch_invite().await?;

    if target.creator() == &user.id {
        target.delete().await?;
    } else {
        match &target {
            Invite::Server { server, .. } => {
                let server = Ref::from_unchecked(server.clone()).fetch_server().await?;
                let perm = permissions::PermissionCalculator::new(&user)
                    .with_server(&server)
                    .for_server()
                    .await?;

                if !perm.get_manage_server() {
                    return Err(Error::MissingPermission);
                }

                target.delete().await?;
            }
            _ => unreachable!(),
        }
    }

    Ok(EmptyResponse {})
}
