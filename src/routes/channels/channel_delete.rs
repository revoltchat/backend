use revolt_quark::{
    models::{channel::PartialChannel, Channel, User},
    perms, ChannelPermission, Db, EmptyResponse, Error, Ref, Result,
};

#[delete("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;
    let perm = perms(&user).channel(&channel).calc_channel(db).await;

    if !perm.get_view() {
        return Err(Error::NotFound);
    }

    match &channel {
        Channel::SavedMessages { .. } => Err(Error::NoEffect),
        Channel::DirectMessage { id, .. } => db
            .update_channel(
                id,
                &PartialChannel {
                    active: Some(false),
                    ..Default::default()
                },
                vec![],
            )
            .await
            .map(|_| EmptyResponse),
        Channel::Group { .. } => channel
            .remove_user_from_group(db, &user.id)
            .await
            .map(|_| EmptyResponse),
        Channel::TextChannel { .. } | Channel::VoiceChannel { .. } => {
            if perm.get_manage_channel() {
                channel.delete(db).await.map(|_| EmptyResponse)
            } else {
                Err(Error::MissingPermission {
                    permission: ChannelPermission::ManageChannel as i32,
                })
            }
        }
    }
}
