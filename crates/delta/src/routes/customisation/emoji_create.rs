use revolt_quark::models::emoji::EmojiParent;
use revolt_quark::models::{Emoji, File, User};
use revolt_quark::{perms, Db, Error, Permission, Result};
use serde::Deserialize;
use validator::Validate;

use crate::util::regex::RE_EMOJI;

use rocket::serde::json::Json;

/// # Emoji Data
#[derive(Validate, Deserialize, JsonSchema)]
pub struct DataCreateEmoji {
    /// Server name
    #[validate(length(min = 1, max = 32), regex = "RE_EMOJI")]
    name: String,
    /// Parent information
    parent: EmojiParent,
    /// Whether the emoji is mature
    #[serde(default)]
    nsfw: bool,
}

/// # Create New Emoji
///
/// Create an emoji by its Autumn upload id.
#[openapi(tag = "Emojis")]
#[put("/emoji/<id>", data = "<data>")]
pub async fn create_emoji(
    db: &Db,
    user: User,
    id: String,
    data: Json<DataCreateEmoji>,
) -> Result<Json<Emoji>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    // Bots cannot manage emojis
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    // Validate we have permission to write into parent
    match &data.parent {
        EmojiParent::Server { id } => {
            let server = db.fetch_server(id).await?;

            // Check for permission
            perms(&user)
                .server(&server)
                .throw_permission(db, Permission::ManageCustomisation)
                .await?;

            // Check that there are no more than 100 emoji
            // ! FIXME: hardcoded upper limit
            let emojis = db.fetch_emoji_by_parent_id(&server.id).await?;
            if emojis.len() > 100 {
                return Err(Error::TooManyEmoji);
            }
        }
        EmojiParent::Detached => return Err(Error::InvalidOperation),
    };

    // Find the relevant attachment
    let attachment = File::use_emoji(db, &id, &id).await?;

    // Create the emoji object
    let emoji = Emoji {
        id,
        parent: data.parent,
        creator_id: user.id,
        name: data.name,
        animated: "image/gif" == &attachment.content_type,
        nsfw: data.nsfw,
    };

    // Save emoji
    emoji.create(db).await?;
    Ok(Json(emoji))
}
