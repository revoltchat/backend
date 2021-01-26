use crate::database::*;
use crate::util::result::{Error, Result};

use rocket_contrib::json::JsonValue;

#[get("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let mut target = target.fetch_user().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_user(&target)
        .for_user_given()
        .await?;

    if !perm.get_access() {
        Err(Error::LabelMe)?
    }

    if user.id != target.id {
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

    Ok(json!(target.with(perm)))
}
