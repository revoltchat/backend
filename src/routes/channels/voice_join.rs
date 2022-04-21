use revolt_quark::{
    models::{Channel, User},
    perms,
    variables::delta::{USE_VOSO, VOSO_MANAGE_TOKEN, VOSO_URL},
    Db, Error, Permission, Ref, Result,
};

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
#[post("/<target>/join_call")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<CreateVoiceUserResponse>> {
    let channel = target.as_channel(db).await?;
    let mut permissions = perms(&user).channel(&channel);

    permissions
        .throw_permission_and_view_channel(db, Permission::Connect)
        .await?;

    if !*USE_VOSO {
        return Err(Error::VosoUnavailable);
    }

    match channel {
        Channel::SavedMessages { .. } | Channel::TextChannel { .. } => {
            return Err(Error::CannotJoinCall)
        }
        _ => {}
    }

    // To join a call:
    // - Check if the room exists.
    // - If not, create it.
    let client = reqwest::Client::new();
    let result = client
        .get(&format!("{}/room/{}", *VOSO_URL, channel.id()))
        .header(
            reqwest::header::AUTHORIZATION,
            VOSO_MANAGE_TOKEN.to_string(),
        )
        .send()
        .await;

    match result {
        Err(_) => return Err(Error::VosoUnavailable),
        Ok(result) => match result.status() {
            reqwest::StatusCode::OK => (),
            reqwest::StatusCode::NOT_FOUND => {
                if (client
                    .post(&format!("{}/room/{}", *VOSO_URL, channel.id()))
                    .header(
                        reqwest::header::AUTHORIZATION,
                        VOSO_MANAGE_TOKEN.to_string(),
                    )
                    .send()
                    .await)
                    .is_err()
                {
                    return Err(Error::VosoUnavailable);
                }
            }
            _ => return Err(Error::VosoUnavailable),
        },
    }

    // Then create a user for the room.
    if let Ok(response) = client
        .post(&format!(
            "{}/room/{}/user/{}",
            *VOSO_URL,
            channel.id(),
            user.id
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            VOSO_MANAGE_TOKEN.to_string(),
        )
        .send()
        .await
    {
        response
            .json()
            .await
            .map_err(|_| Error::InvalidOperation)
            .map(Json)
    } else {
        Err(Error::VosoUnavailable)
    }
}
