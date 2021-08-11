use crate::database::*;
use crate::util::result::{Error, Result};

use rocket::serde::json::Value;

#[get("/<target>/relationship")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }
    
    Ok(json!({ "status": get_relationship(&user, &target.id) }))
}
