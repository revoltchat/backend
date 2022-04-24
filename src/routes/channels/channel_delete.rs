use revolt_quark::{
    models::{channel::PartialChannel, Channel, User},
    perms, Db, EmptyResponse, Error, Permission, Ref, Result,
};

/// # Close Channel
///
/// Deletes a server channel, leaves a group or closes a group.
#[openapi(tag = "Channel Information")]
#[delete("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    let mut channel = target.as_channel(db).await?;
    let mut perms = perms(&user).channel(&channel);
    perms.throw_permission(db, Permission::ViewChannel).await?;

    match &channel {
        Channel::SavedMessages { .. } => Err(Error::NoEffect),
        Channel::DirectMessage { .. } => channel
            .update(
                db,
                PartialChannel {
                    active: Some(false),
                    ..Default::default()
                },
                vec![],
            )
            .await
            .map(|_| EmptyResponse),
        Channel::Group { .. } => channel
            .remove_user_from_group(db, &user.id, None)
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
