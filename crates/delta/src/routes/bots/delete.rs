use revolt_database::{util::reference::Reference, Database, User};
use revolt_result::{create_error, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Delete Bot
///
/// Delete a bot by its id.
#[openapi(tag = "Bots")]
#[delete("/<target>")]
pub async fn delete_bot(
    db: &State<Database>,
    user: User,
    target: Reference,
) -> Result<EmptyResponse> {
    let bot = target.as_bot(db).await?;
    if bot.owner != user.id {
        return Err(create_error!(NotFound));
    }

    bot.delete(db).await.map(|_| EmptyResponse)
}
