use crate::database::*;
use crate::util::result::{Error, Result};

use rocket_contrib::json::JsonValue;

#[get("/<target>/messages/<msg>")]
pub async fn req(user: User, target: Ref, msg: Ref) -> Result<JsonValue> {
    let channel = target.fetch_channel().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&channel)
        .for_channel().await?;
    if !perm.get_view() {
        Err(Error::LabelMe)?
    }

    let message = msg.fetch_message(&channel).await?;
    Ok(json!(message))
}
