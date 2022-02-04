use revolt_quark::models::User;
use revolt_quark::{Database, Result};

use mongodb::bson::doc;
use rocket::serde::json::Json;
use rocket::State;

#[put("/<username>/friend")]
pub async fn req(db: &State<Database>, user: User, username: String) -> Result<Json<User>> {
    let mut target = db.fetch_user_by_username(&username).await?;
    user.add_friend(db, &mut target).await?;
    Ok(Json(target.with_auto_perspective(db, &user).await))
}
