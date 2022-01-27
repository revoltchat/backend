use revolt_quark::Result;

use rocket::serde::json::Value;

#[get("/<target>/messages/<msg>")]
pub async fn req(/*user: UserRef, target: Ref, msg: Ref*/ target: String, msg: String) -> Result<Value> {
    todo!()
}
