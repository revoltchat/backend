use revolt_quark::{models::User, Ref, Result};

use rocket::serde::json::Value;
use serde::{Deserialize, Serialize};

/// # Token Response
#[derive(Serialize, Deserialize)]
struct CreateVoiceUserResponse {
    /// Token for authenticating with the voice server
    token: String,
}

/// # Join Call
///
/// Asks the voice server for a token to join the call.
#[openapi(tag = "Voice")]
#[post("/<_target>/join_call")]
pub async fn req(_user: User, _target: Ref) -> Result<Value> {
    unimplemented!()
}
