use revolt_quark::{Ref, Result, models::User};

use rocket::serde::json::Value;

#[get("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
    todo!()
}
