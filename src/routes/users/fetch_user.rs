use crate::{database::*, notifications::websocket::is_online};
use crate::util::result::{Error, Result};

use rocket_contrib::json::JsonValue;

#[get("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let mut target = target.fetch_user().await?;

    if user.id != target.id {
        // Check whether we are allowed to fetch this user.
        let perm = permissions::user::calculate(&user, &target.id).await;
        if !perm.get_access() {
            Err(Error::LabelMe)?
        }

        // Only return user relationships if the target is the caller.
        target.relations = None;

        // Add relevant relationship
        if let Some(relationships) = &user.relations {
            target.relationship = relationships
                .iter()
                .find(|x| x.id == user.id)
                .map(|x| x.status.clone())
                .or_else(|| Some(RelationshipStatus::None));
        } else {
            target.relationship = Some(RelationshipStatus::None);
        }
    } else {
        target.relationship = Some(RelationshipStatus::User);
    }

    target.online = Some(is_online(&target.id));

    Ok(json!(target))
}
