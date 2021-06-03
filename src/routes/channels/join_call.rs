use crate::database::*;
use crate::util::result::{Error, Result};
use crate::util::variables::{USE_VOSO, VOSO_MANAGE_TOKEN, VOSO_URL};

use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CreateUserResponse {
    token: String,
}

#[post("/<target>/join_call")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    if !*USE_VOSO {
        return Err(Error::VosoUnavailable);
    }

    let target = target.fetch_channel().await?;
    match target {
        Channel::SavedMessages { .. } | Channel::TextChannel { .. } => {
            return Err(Error::CannotJoinCall)
        }
        _ => {}
    }

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel()
        .await?;

    if !perm.get_voice_call() {
        return Err(Error::MissingPermission);
    }

    // To join a call:
    // - Check if the room exists.
    // - If not, create it.
    let client = reqwest::Client::new();
    let result = client
        .get(&format!("{}/room/{}", *VOSO_URL, target.id()))
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
                if let Err(_) = client
                    .post(&format!("{}/room/{}", *VOSO_URL, target.id()))
                    .header(
                        reqwest::header::AUTHORIZATION,
                        VOSO_MANAGE_TOKEN.to_string(),
                    )
                    .send()
                    .await
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
            target.id(),
            user.id
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            VOSO_MANAGE_TOKEN.to_string(),
        )
        .send()
        .await
    {
        let res: CreateUserResponse = response.json().await.map_err(|_| Error::InvalidOperation)?;

        Ok(json!(res))
    } else {
        Err(Error::VosoUnavailable)
    }
}
