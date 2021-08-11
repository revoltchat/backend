use crate::database::*;
use crate::util::result::{Error, Result};

use rocket::serde::json::Value;

#[get("/relationships")]
pub async fn req(user: User) -> Result<Value> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }
    
    Ok(if let Some(vec) = user.relations {
        json!(vec)
    } else {
        json!([])
    })
}
