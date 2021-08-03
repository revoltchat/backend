use crate::database::*;
use crate::util::result::{Error, Result};

use rocket::serde::json::Value;

#[get("/<target>/messages/<msg>")]
pub async fn req(user: User, target: Ref, msg: Ref) -> Result<Value> {
    let channel = target.fetch_channel().await?;
    channel.has_messaging()?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&channel)
        .for_channel()
        .await?;
    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    let message = msg.fetch_message(&channel).await?;
    Ok(json!(message))
}
