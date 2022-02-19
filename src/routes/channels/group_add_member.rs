use revolt_quark::{
    models::{Channel, User},
    perms, ChannelPermission, Db, EmptyResponse, Error, Ref, Result,
};

#[put("/<target>/recipients/<member>")]
pub async fn req(db: &Db, user: User, target: Ref, member: Ref) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;
    if !perms(&user)
        .channel(&channel)
        .calc_channel(db)
        .await
        .get_invite_others()
    {
        return Err(Error::MissingPermission {
            permission: ChannelPermission::InviteOthers as i32,
        });
    }

    match &channel {
        Channel::Group { .. } => {
            let member = member.as_user(db).await?;
            channel
                .add_user_to_group(db, &member.id, &user.id)
                .await
                .map(|_| EmptyResponse)
        }
        _ => Err(Error::InvalidOperation),
    }
}
