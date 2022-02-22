use revolt_quark::{
    models::{channel::PartialChannel, Channel, User},
    perms, Db, EmptyResponse, Error, Permission, Ref, Result,
};

#[delete("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;
    let perm = perms(&user).channel(&channel).calc(db).await;

    if !perm.can_view_channel() {
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
            if perm.can_manage_channel() {
                channel.delete(db).await.map(|_| EmptyResponse)
            } else {
                Error::from_permission(Permission::ManageChannel)
            }
        }
    }
}
