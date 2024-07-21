use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference}, Database, PartialMessage, User
};
use revolt_permissions::{calculate_channel_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Unpins a message
///
/// Unpins a message by its id.
#[openapi(tag = "Messaging")]
#[post("/<target>/messages/<msg>/unpin")]
pub async fn message_unpin(
    db: &State<Database>,
    user: User,
    target: Reference,
    msg: Reference,
) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;

    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageMessages)?;

    let mut message = msg.as_message_in_channel(db, &channel.id()).await?;

    if !message.pinned.unwrap_or_default() {
        return Err(create_error!(NotPinned))
    }

    message.update(db, PartialMessage {
        pinned: Some(false),
        ..Default::default()
    }, vec![FieldsMessage::Pinned]).await?;

    Ok(EmptyResponse)
}
