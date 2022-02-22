use revolt_quark::{models::User, perms, Db, EmptyResponse, Error, Permission, Ref, Result};

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
                .has_permission(db, Permission::ManageMessages)
                .await?
        }
    {
        return Error::from_permission(Permission::ManageMessages);
    }

    message.delete(db).await.map(|_| EmptyResponse)
}
