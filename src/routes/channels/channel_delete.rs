use crate::util::result::{Error, Result, EmptyResponse};
use crate::{database::*, notifications::events::ClientboundNotification};

use mongodb::bson::doc;

#[delete("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<EmptyResponse> {
    let target = target.fetch_channel().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel()
        .await?;

    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    match &target {
        Channel::SavedMessages { .. } => Err(Error::NoEffect),
        Channel::DirectMessage { .. } => {
            db_conn().make_channel_inactive(&target.id()).await?;
            Ok(EmptyResponse {})
        }
        Channel::Group {
            id,
            owner,
            recipients,
            ..
        } => {
            if &user.id == owner {
                if let Some(new_owner) = recipients.iter().find(|x| *x != &user.id) {
                    db_conn()
                        .update_channel_owner(&id, new_owner, &user.id)
                        .await?;
                    target.publish_update(json!({ "owner": new_owner })).await?;
                } else {
                    target.delete().await?;
                    return Ok(EmptyResponse {});
                }
            } else {
                db_conn()
                    .remove_recipient_from_channel(&id, &user.id)
                    .await?;
            }

            ClientboundNotification::ChannelGroupLeave {
                id: id.clone(),
                user: user.id.clone(),
            }
            .publish(id.clone());

            Content::SystemMessage(SystemMessage::UserLeft { id: user.id })
                .send_as_system(&target)
                .await
                .ok();

            Ok(EmptyResponse {})
        }
        Channel::TextChannel { .. } |
        Channel::VoiceChannel { .. } => {
            if perm.get_manage_channel() {
                target.delete().await?;
                Ok(EmptyResponse {})
            } else {
                Err(Error::MissingPermission)
            }
        }
    }
}
