use std::collections::HashSet;

use revolt_quark::models::snapshot::{SnapshotContent, SnapshotWithContext};
use revolt_quark::models::{Channel, User};
use revolt_quark::{Db, Error, Result};
use rocket::serde::json::Json;

/// # Fetch Snapshot
///
/// Fetch a snapshot for a given report
#[openapi(tag = "User Safety")]
#[get("/snapshot/<report_id>")]
pub async fn fetch_snapshot(
    db: &Db,
    user: User,
    report_id: String,
) -> Result<Json<SnapshotWithContext>> {
    // Must be privileged for this route
    if !user.privileged {
        return Err(Error::NotPrivileged);
    }

    // Fetch snapshot
    let snapshot = db.fetch_snapshot(&report_id).await?;

    // Resolve and fetch IDs of associated content
    let mut user_ids: HashSet<&str> = HashSet::new();
    let mut channel_ids: HashSet<&str> = HashSet::new();

    match &snapshot.content {
        SnapshotContent::Message {
            prior_context,
            leading_context,
            message,
        } => {
            for msg in prior_context {
                user_ids.insert(&msg.author);
            }

            for msg in leading_context {
                user_ids.insert(&msg.author);
            }

            user_ids.insert(&message.author);
            channel_ids.insert(&message.channel);
        }
        _ => {
            todo!()
        }
    }

    // Collect user and channel IDs
    let user_ids: Vec<String> = user_ids.into_iter().map(|s| s.to_owned()).collect();
    let channel_ids: Vec<String> = channel_ids.into_iter().map(|s| s.to_owned()).collect();

    // Fetch users and channels
    let users = db.fetch_users(&user_ids).await?;
    let channels = db.fetch_channels(&channel_ids).await?;

    // Pull out first server from channels if possible
    let server = if let Some(server_id) = channels.iter().find_map(|channel| match channel {
        Channel::TextChannel { server, .. } => Some(server),
        _ => None,
    }) {
        Some(db.fetch_server(server_id).await?)
    } else {
        None
    };

    // Return snapshot with context
    Ok(Json(SnapshotWithContext {
        snapshot,
        users,
        channels,
        server,
    }))
}
