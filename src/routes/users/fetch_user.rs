use crate::database::*;
use crate::util::result::{Error, Result};

use rocket_contrib::json::JsonValue;

#[get("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let target = target.fetch_user().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_user(&target)
        .for_user_given()
        .await?;

    if !perm.get_access() {
        Err(Error::LabelMe)?
    }

    Ok(json!(target.from(&user).with(perm)))
}
