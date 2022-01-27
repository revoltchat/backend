use revolt_quark::Result;

use rocket::serde::json::Value;

#[get("/@me")]
pub async fn req(/*user: UserRef*/) -> Result<Value> {
    todo!()
}
