use revolt_quark::{models::User, perms, Db, EmptyResponse, Error, Ref, Result};

#[put("/<target>/ack/<message>")]
pub async fn req(db: &Db, user: User, target: Ref, message: Ref) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;
    if !perms(&user)
        .channel(&channel)
        .calc_channel(db)
        .await
        .get_view()
    {
        return Err(Error::NotFound);
    }

    db.acknowledge_message(channel.id(), &user.id, &message.id)
        .await
        .map(|_| EmptyResponse)
}
