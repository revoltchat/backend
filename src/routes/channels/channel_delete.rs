use revolt_quark::{
    models::{channel::PartialChannel, Channel, User},
    perms, Db, EmptyResponse, Error, Permission, Ref, Result,
};

#[delete("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;
    let mut perms = perms(&user).channel(&channel);
    perms.throw_permission(db, Permission::ViewChannel).await?;

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
            perms
                .throw_permission(db, Permission::ManageChannel)
                .await?;

            channel.delete(db).await.map(|_| EmptyResponse)
        }
    }
}
