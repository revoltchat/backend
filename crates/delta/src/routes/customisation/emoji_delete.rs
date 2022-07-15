use revolt_quark::models::{emoji::EmojiParent, User};
use revolt_quark::{perms, Db, EmptyResponse, Error, Permission, Ref, Result};

/// # Delete Emoji
///
/// Delete an emoji by its id.
#[openapi(tag = "Emojis")]
#[delete("/emoji/<id>")]
pub async fn delete_emoji(db: &Db, user: User, id: Ref) -> Result<EmptyResponse> {
    // Bots cannot manage emoji
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    // Fetch the emoji
    let emoji = id.as_emoji(db).await?;

    // If we uploaded the emoji, then we have permission to delete it
    if emoji.creator_id != user.id {
        // Otherwise, validate we have permission to delete from parent
        match &emoji.parent {
            EmojiParent::Server { id } => {
                let server = db.fetch_server(id).await?;

                // Check for permission
                perms(&user)
                    .server(&server)
                    .throw_permission(db, Permission::ManageCustomisation)
                    .await?;
            }
            EmojiParent::Detached => return Ok(EmptyResponse),
        };
    }

    // Delete the emoji
    emoji.delete(db).await.map(|_| EmptyResponse)
}
