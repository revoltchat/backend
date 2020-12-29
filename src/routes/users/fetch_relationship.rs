use crate::database::{entities::User, guards::reference::Ref, permissions::get_relationship};
use rocket_contrib::json::JsonValue;
use crate::util::result::Result;

#[get("/<target>/relationship")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    Ok(json!({
        "status": get_relationship(&user, &target)
    }))
}
