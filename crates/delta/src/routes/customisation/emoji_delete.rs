use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    Database, EmojiParent, User,
};
use revolt_permissions::{calculate_server_permissions, ChannelPermission};
use revolt_result::{create_error, Result};

use rocket::State;
use rocket_empty::EmptyResponse;

/// # Delete Emoji
///
/// Delete an emoji by its id.
#[openapi(tag = "Emojis")]
#[delete("/emoji/<emoji_id>")]
pub async fn delete_emoji(
    db: &State<Database>,
    user: User,
    emoji_id: Reference,
) -> Result<EmptyResponse> {
    // Bots cannot manage emoji
    if user.bot.is_some() {
        return Err(create_error!(IsBot));
    }

    // Fetch the emoji
    let emoji = emoji_id.as_emoji(db).await?;

    // If we uploaded the emoji, then we have permission to delete it
    if emoji.creator_id != user.id {
        // Otherwise, validate we have permission to delete from parent
        match &emoji.parent {
            EmojiParent::Server { id } => {
                let server = db.fetch_server(id).await?;

                // Check for permission
                let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
                calculate_server_permissions(&mut query)
                    .await
                    .throw_if_lacking_channel_permission(ChannelPermission::ManageCustomisation)?;
            }
            EmojiParent::Detached => return Ok(EmptyResponse),
        };
    }

    // Delete the emoji
    emoji.delete(db).await.map(|_| EmptyResponse)
}
