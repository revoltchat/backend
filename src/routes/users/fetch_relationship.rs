use crate::database::*;
use crate::util::result::Result;

use rocket::serde::json::Value;

#[get("/<target>/relationship")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
    Ok(json!({ "status": get_relationship(&user, &target.id) }))
}
