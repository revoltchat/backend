use crate::database::{entities::User, guards::reference::Ref, permissions::get_relationship};
use crate::util::result::Result;
use rocket_contrib::json::JsonValue;

#[get("/<target>/relationship")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    Ok(json!({ "status": get_relationship(&user, &target) }))
}
