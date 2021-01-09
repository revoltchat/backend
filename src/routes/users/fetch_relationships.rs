use crate::database::*;
use crate::util::result::Result;

use rocket_contrib::json::JsonValue;

#[get("/relationships")]
pub async fn req(user: User) -> Result<JsonValue> {
    Ok(if let Some(vec) = user.relations {
        json!(vec)
    } else {
        json!([])
    })
}
