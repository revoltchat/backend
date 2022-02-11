use revolt_quark::{models::User, perms, ChannelPermission, Db, EmptyResponse, Error, Ref, Result};

#[delete("/<target>/messages/<msg>")]
pub async fn req(db: &Db, user: User, target: Ref, msg: Ref) -> Result<EmptyResponse> {
    let message = msg.as_message(db).await?;
    if message.channel != target.id {
        return Err(Error::NotFound);
    }

    if message.author != user.id
        || !{
            perms(&user)
                .channel(&target.as_channel(db).await?)
                .calc_channel(db)
                .await
                .get_manage_messages()
        }
    {
        return Err(Error::MissingPermission {
            permission: ChannelPermission::ManageMessages as i32,
        });
    }

    db.delete_message(&message.id).await.map(|_| EmptyResponse)
}
