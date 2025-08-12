use revolt_config::config;
use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    Channel, Database, User,
};
use revolt_models::v0;
use revolt_permissions::{calculate_channel_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};

/// # Join Call
///
/// Asks the voice server for a token to join the call.
#[openapi(tag = "Voice")]
#[post("/<target>/join_call")]
pub async fn call(
    db: &State<Database>,
    user: User,
    target: Reference<'_>,
) -> Result<Json<v0::LegacyCreateVoiceUserResponse>> {
    let channel = target.as_channel(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::Connect)?;

    let config = config().await;
    if config.api.security.voso_legacy_token.is_empty() {
        return Err(create_error!(VosoUnavailable));
    }

    match channel {
        Channel::SavedMessages { .. } | Channel::TextChannel { .. } => {
            return Err(create_error!(CannotJoinCall))
        }
        _ => {}
    }

    // To join a call:
    // - Check if the room exists.
    // - If not, create it.
    let client = reqwest::Client::new();
    let result = client
        .get(format!(
            "{}/room/{}",
            config.hosts.voso_legacy,
            channel.id()
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            config.api.security.voso_legacy_token.clone(),
        )
        .send()
        .await;

    match result {
        Err(_) => return Err(create_error!(VosoUnavailable)),
        Ok(result) => match result.status() {
            reqwest::StatusCode::OK => (),
            reqwest::StatusCode::NOT_FOUND => {
                if (client
                    .post(format!(
                        "{}/room/{}",
                        config.hosts.voso_legacy,
                        channel.id()
                    ))
                    .header(
                        reqwest::header::AUTHORIZATION,
                        config.api.security.voso_legacy_token.clone(),
                    )
                    .send()
                    .await)
                    .is_err()
                {
                    return Err(create_error!(VosoUnavailable));
                }
            }
            _ => return Err(create_error!(VosoUnavailable)),
        },
    }

    // Then create a user for the room.
    if let Ok(response) = client
        .post(format!(
            "{}/room/{}/user/{}",
            config.hosts.voso_legacy,
            channel.id(),
            user.id
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            config.api.security.voso_legacy_token,
        )
        .send()
        .await
    {
        response
            .json()
            .await
            .map_err(|_| create_error!(InvalidOperation))
            .map(Json)
    } else {
        Err(create_error!(VosoUnavailable))
    }
}
