use revolt_quark::{
    models::{channel::PartialChannel, Channel, User},
    perms, ChannelPermission, Db, EmptyResponse, Error, Ref, Result,
};

use mongodb::bson::doc;

#[delete("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;
    let perm = perms(&user).channel(&channel).calc_channel(db).await;

    if !perm.get_view() {
        return Err(Error::NotFound);
    }

    match channel {
        Channel::SavedMessages { .. } => Err(Error::NoEffect),
        Channel::DirectMessage { id, .. } => {
            db.update_channel(
                &id,
                &PartialChannel {
                    active: Some(false),
                    ..Default::default()
                },
                vec![],
            )
            .await?;

            Ok(EmptyResponse)
        }
        Channel::Group {
            id,
            owner,
            recipients,
            ..
        } => {
            if user.id == owner {
                if let Some(new_owner) = recipients.iter().find(|x| *x != &user.id) {
                    db.update_channel(
                        &id,
                        &PartialChannel {
                            owner: Some(new_owner.into()),
                            ..Default::default()
                        },
                        vec![],
                    )
                    .await?;
                } else {
                    db.delete_channel(&id).await?;
                    return Ok(EmptyResponse);
                }
            }

            db.remove_user_from_group(&id, &user.id).await?;

            /*ClientboundNotification::ChannelGroupLeave {
                id: id.clone(),
                user: user.id.clone(),
            }
            .publish(id.clone());

            Content::SystemMessage(SystemMessage::UserLeft { id: user.id })
                .send_as_system(&target)
                .await
                .ok();*/

            Ok(EmptyResponse)
        }
        Channel::TextChannel { id, .. } | Channel::VoiceChannel { id, .. } => {
            if perm.get_manage_channel() {
                db.delete_channel(&id).await?;
                Ok(EmptyResponse)
            } else {
                Err(Error::MissingPermission {
                    permission: ChannelPermission::ManageChannel as i32,
                })
            }
        }
    }
}
