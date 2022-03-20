use revolt_quark::{models::User, Ref, Result};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

/// # Voice Server Token Response
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct CreateVoiceUserResponse {
    /// Token for authenticating with the voice server
    token: String,
}

/// # Join Call
///
/// Asks the voice server for a token to join the call.
#[openapi(tag = "Voice")]
#[post("/<_target>/join_call")]
pub async fn req(_user: User, _target: Ref) -> Result<Json<CreateVoiceUserResponse>> {
    unimplemented!()
}
