use crate::database::*;
use crate::util::result::{Error, Result};

use rocket_contrib::json::JsonValue;

#[get("/<target>/messages/<msg>")]
pub async fn req(user: User, target: Ref, msg: Ref) -> Result<JsonValue> {
    let channel = target.fetch_channel().await?;

    let perm = permissions::channel::calculate(&user, &channel).await;
    if !perm.get_view() {
        Err(Error::LabelMe)?
    }

    let message = msg.fetch_message().await?;
    Ok(json!(message))
}
