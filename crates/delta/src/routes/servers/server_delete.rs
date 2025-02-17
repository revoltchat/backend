use revolt_database::{
    util::reference::Reference,
    voice::{delete_voice_state, get_channel_node, get_user_voice_channel_in_server, get_voice_channel_members, VoiceClient},
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
            if let Some(users) = get_voice_channel_members(channel_id).await? {
                let node = get_channel_node(channel_id).await?;

                for user in users {
                    voice_client.remove_user(&node, &user, channel_id).await?;
                    delete_voice_state(channel_id, Some(&server.id), &user).await?;
                }
            }
        }

        server.delete(db).await
    } else {
        if let Some(channel_id) = get_user_voice_channel_in_server(&user.id, &server.id).await? {
            if server.channels.iter().any(|c| c == &channel_id) {
                let node = get_channel_node(&channel_id).await?;
                voice_client.remove_user(&node, &user.id, &channel_id).await?;
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
