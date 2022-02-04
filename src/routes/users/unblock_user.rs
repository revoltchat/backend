use revolt_quark::models::User;
use revolt_quark::{Result, Database};

use mongodb::bson::doc;
use rocket::State;
use rocket::serde::json::Json;

#[delete("/<username>/block")]
pub async fn req(db: &State<Database>, user: User, username: String) -> Result<Json<User>> {
    let mut target = db.fetch_user_by_username(&username).await?;
    user.unblock_user(db, &mut target).await?;
    Ok(Json(target.with_auto_perspective(db, &user).await))
}
