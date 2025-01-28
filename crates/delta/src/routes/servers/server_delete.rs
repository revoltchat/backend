use revolt_database::{
    util::reference::Reference,
    voice::{delete_voice_state, get_user_voice_channel_in_server, VoiceClient},
    Database, RemovalIntention, User,
};
use revolt_models::v0;
use revolt_result::Result;
use rocket::State;

use rocket_empty::EmptyResponse;

/// # Delete / Leave Server
///
/// Deletes a server if owner otherwise leaves.
#[openapi(tag = "Server Information")]
#[delete("/<target>?<options..>")]
pub async fn delete(
    db: &State<Database>,
    voice_client: &State<VoiceClient>,
    user: User,
    target: Reference,
    options: v0::OptionsServerDelete,
) -> Result<EmptyResponse> {
    let server = target.as_server(db).await?;
    let member = db.fetch_member(&target.id, &user.id).await?;

    if server.owner == user.id {
        for channel_id in &server.channels {
            voice_client.remove_user(&user, channel_id).await?;
            delete_voice_state(channel_id, Some(&server.id), &user.id).await?;
        }

        server.delete(db).await
    } else {
        if let Some(channel_id) = get_user_voice_channel_in_server(&user.id, &server.id).await? {
            if server.channels.iter().any(|c| c == &channel_id) {
                voice_client.remove_user(&user, &channel_id).await?;
                delete_voice_state(&channel_id, Some(&server.id), &user.id).await?;
            }
        };

        member
            .remove(
                db,
                &server,
                RemovalIntention::Leave,
                options.leave_silently.unwrap_or_default(),
            )
            .await
    }
    .map(|_| EmptyResponse)
}
