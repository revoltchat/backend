use crate::database::*;
use crate::util::result::{Error, Result};

use rocket::serde::json::Value;

#[get("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
    let mut target = target.fetch_user().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_user(&target)
        .for_user_given()
        .await?;

    if !perm.get_access() {
        Err(Error::MissingPermission)?
    }

    // If the user's status is `Presence::Invisible`, return it as `Presence::Offline`
    if let Some(mut status) = target.status {
        if let Some(presence) = status.presence {
            if presence == Presence::Invisible {
                status.presence = Some(Presence::Offline);
                target.status = Some(status);
            }
        }
    }

    Ok(json!(target.from(&user).with(perm)))
}
