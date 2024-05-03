use revolt_quark::{models::User, perms, Db, EmptyResponse, Permission, Ref, Result};
use serde::{Deserialize, Serialize};

/// # Query Parameters
#[derive(Serialize, Deserialize, JsonSchema, FromForm)]
pub struct OptionsUnreact {
    /// Remove a specific user's reaction
    user_id: Option<String>,
    /// Remove all reactions
    remove_all: Option<bool>,
}

/// # Remove Reaction(s) to Message
///
/// Remove your own, someone else's or all of a given reaction.
///
/// Requires `ManageMessages` if changing others' reactions.
#[openapi(tag = "Interactions")]
#[delete("/<target>/messages/<msg>/reactions/<emoji>?<options..>")]
pub async fn unreact_message(
    db: &Db,
    user: User,
    target: Ref,
    msg: Ref,
    emoji: Ref,
    options: OptionsUnreact,
) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;
    let mut permissions = perms(&user).channel(&channel);
    permissions
        .throw_permission_and_view_channel(db, Permission::React)
        .await?;

    // Check if we need to escalate permissions
    let remove_all = options.remove_all.unwrap_or_default();
    if options.user_id.is_some() || remove_all {
        permissions
            .throw_permission(db, Permission::ManageMessages)
            .await?;
    }

    // Fetch relevant message
    let message = msg.as_message_in(db, channel.id()).await?;

    // Check if we should wipe all of this reaction
    if remove_all {
        return message
            .clear_reaction(db, &emoji.id)
            .await
            .map(|_| EmptyResponse);
    }

    // Remove the reaction
    message
        .remove_reaction(db, options.user_id.as_ref().unwrap_or(&user.id), &emoji.id)
        .await
        .map(|_| EmptyResponse)
}
