use revolt_quark::{models::User, perms, Db, EmptyResponse, Permission, Ref, Result};

/// # Add Reaction to Message
///
/// React to a given message.
#[openapi(tag = "Interactions")]
#[put("/<target>/messages/<msg>/reactions/<emoji>")]
pub async fn react_message(
    db: &Db,
    user: User,
    target: Ref,
    msg: Ref,
    emoji: Ref,
) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission_and_view_channel(db, Permission::React)
        .await?;

    // Fetch relevant message
    let message = msg.as_message_in(db, channel.id()).await?;

    // Add the reaction
    message
        .add_reaction(db, &user, &emoji.id)
        .await
        .map(|_| EmptyResponse)
}
