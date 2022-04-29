use revolt_quark::{models::User, Ref, Result};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// # Query Parameters
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct OptionsQueryStale {
    /// Array of message IDs
    #[validate(length(min = 0, max = 150))]
    ids: Vec<String>,
}

/// # Poll Message Changes
///
/// This route returns any changed message objects and tells you if any have been deleted.
///
/// Don't actually poll this route, instead use this to update your local database.
///
/// **DEPRECATED**
#[openapi(tag = "Messaging")]
#[post("/<_target>/messages/stale", data = "<_data>")]
pub async fn req(_user: User, _target: Ref, _data: Json<OptionsQueryStale>) -> Result<()> {
    Ok(())
}
