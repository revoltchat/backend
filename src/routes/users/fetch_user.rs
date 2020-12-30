use crate::database::entities::User;
use crate::database::guards::reference::Ref;
use crate::util::result::{Error, Result};
use rocket_contrib::json::JsonValue;

#[get("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let mut target = target.fetch_user().await?;

    if user.id != target.id {
        // Check whether we are allowed to fetch this user.
        let perm = crate::database::permissions::temp_calc_perm(&user, &target).await;
        if !perm.get_access() {
            Err(Error::LabelMe)?
        }

        // Only return user relationships if the target is the caller.
        target.relations = None;
    }

    Ok(json!(target))
}
