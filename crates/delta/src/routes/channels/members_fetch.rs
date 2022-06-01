use revolt_quark::{
    models::{Channel, User},
    perms, Db, Error, Permission, Ref, Result,
};

use rocket::serde::json::Json;

/// # Fetch Group Members
///
/// Retrieves all users who are part of this group.
#[openapi(tag = "Groups")]
#[get("/<target>/members")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Vec<User>>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission(db, Permission::ViewChannel)
        .await?;

    if let Channel::Group { recipients, .. } = channel {
        Ok(Json(
            db.fetch_users(&recipients)
                .await?
                .into_iter()
                .map(|x| x.with_relationship(&user))
                .collect::<Vec<User>>(),
        ))
    } else {
        Err(Error::InvalidOperation)
    }
}
