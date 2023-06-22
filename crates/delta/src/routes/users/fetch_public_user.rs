use revolt_quark::{models::User, Database, Error, Ref, Result, UserPermission, UserPermissions};

use rocket::{serde::json::Json, State};

/// # Fetch Public User
///
/// Retrieve some public user information. This endpoint can only be used by bots
#[openapi(tag = "User Information")]
#[get("/<target>/public")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<User>> {
    if target.id == user.id {
        return Ok(Json(user));
    }

    // Only bots can use this endpoint at this time
    //
    // This is to allow for counter-abuse measures to be put in place
    if user.bot.is_none() {
        return Err(Error::IsNotBot);
    }

    let target = target.as_user(db).await?;

    // We treat the bot as a user that can merely access the user
    let permissions = UserPermissions([UserPermission::Access as u32]);

    Ok(Json(target.with_perspective(&user, &permissions)))
}
