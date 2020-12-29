use crate::database::entities::User;
use rocket_contrib::json::JsonValue;
use crate::util::result::Result;

#[get("/relationships")]
pub async fn req(user: User) -> Result<JsonValue> {
    Ok(
        if let Some(vec) = user.relations {
            json!(vec)
        } else {
            json!([])
        }
    )
}
