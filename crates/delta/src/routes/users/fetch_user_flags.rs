use revolt_quark::{Database, Ref, Result};

use rocket::{serde::json::Json, State};
use serde::Serialize;

/// # Flag Response
#[derive(Serialize, JsonSchema)]
pub struct FlagResponse {
    /// Flags
    flags: i32,
}

/// # Fetch User Flags
///
/// Retrieve a user's flags.
#[openapi(tag = "User Information")]
#[get("/<target>/flags")]
pub async fn fetch_user_flags(db: &State<Database>, target: Ref) -> Result<Json<FlagResponse>> {
    let flags = if let Ok(target) = target.as_user(db).await {
        target.flags.unwrap_or_default()
    } else {
        0
    };

    Ok(Json(FlagResponse { flags }))
}
