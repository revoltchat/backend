use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference}, Database, PartialMessage, User
};
use revolt_permissions::{calculate_channel_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Pins a message
///
/// Pins a message by its id.
#[openapi(tag = "Messaging")]
#[post("/<target>/messages/<msg>/pin")]
pub async fn message_pin(
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

    if message.pinned.unwrap_or_default() {
        return Err(create_error!(AlreadyPinned))
    }

    message.update(db, PartialMessage {
        pinned: Some(true),
        ..Default::default()
    }, vec![]).await?;

    Ok(EmptyResponse)
}
