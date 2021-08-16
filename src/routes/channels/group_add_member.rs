use crate::util::result::{Error, Result, EmptyResponse};
use crate::database::*;

use mongodb::bson::doc;

#[put("/<target>/recipients/<member>")]
pub async fn req(user: User, target: Ref, member: Ref) -> Result<EmptyResponse> {
    if get_relationship(&user, &member.id) != RelationshipStatus::Friend {
        Err(Error::NotFriends)?
    }

    let channel = target.fetch_channel().await?;
    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&channel)
        .for_channel()
        .await?;

    if !perm.get_invite_others() {
        Err(Error::MissingPermission)?
    }

    channel.add_to_group(member.id, user.id).await;
    Ok(EmptyResponse {})
}
