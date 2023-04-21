use std::collections::HashSet;

use redis_kiss::{get_connection, AsyncCommands};

use rand::Rng;
mod entry;
mod operations;

use operations::{
    __add_to_set_string, __add_to_set_u32, __delete_key, __get_set_members_as_string,
    __get_set_size, __remove_from_set_string, __remove_from_set_u32,
};

use self::entry::{ONLINE_SET, REGION_KEY};

/// Create a new presence session, returns the ID of this session
pub async fn presence_create_session(user_id: &str, flags: u8) -> (bool, u32) {
    info!("Creating a presence session for {user_id} with flags {flags}");

    if let Ok(mut conn) = get_connection().await {
        // Check whether this is the first session
        let was_empty = __get_set_size(&mut conn, user_id).await == 0;

        // A session ID is comprised of random data and any flags ORed to the end
        let session_id = {
            let mut rng = rand::thread_rng();
            (rng.gen::<u32>() ^ 1) | (flags as u32 & 1)
        };

        // Add session to user's sessions and to the region
        __add_to_set_u32(&mut conn, user_id, session_id).await;
        __add_to_set_string(&mut conn, ONLINE_SET, user_id).await;
        __add_to_set_string(&mut conn, &REGION_KEY, &format!("{user_id}:{session_id}")).await;
        info!("Created session for {user_id}, assigned them a session ID of {session_id}.");

        (was_empty, session_id)
    } else {
        // Fail through
        (false, 0)
    }
}

/// Delete existing presence session
pub async fn presence_delete_session(user_id: &str, session_id: u32) -> bool {
    presence_delete_session_internal(user_id, session_id, false).await
}

/// Delete existing presence session (but also choose whether to skip region)
async fn presence_delete_session_internal(
    user_id: &str,
    session_id: u32,
    skip_region: bool,
) -> bool {
    info!("Deleting presence session for {user_id} with id {session_id}");

    if let Ok(mut conn) = get_connection().await {
        // Remove the session
        __remove_from_set_u32(&mut conn, user_id, session_id).await;

        // Remove from the region
        if !skip_region {
            __remove_from_set_string(&mut conn, &REGION_KEY, &format!("{user_id}:{session_id}"))
                .await;
        }

        // Return whether this was the last session
        let is_empty = __get_set_size(&mut conn, user_id).await == 0;
        if is_empty {
            __remove_from_set_string(&mut conn, ONLINE_SET, user_id).await;
            info!("User ID {} just went offline.", &user_id);
        }

        is_empty
    } else {
        // Fail through
        false
    }
}

/// Check whether a given user ID is online
pub async fn presence_is_online(user_id: &str) -> bool {
    if let Ok(mut conn) = get_connection().await {
        conn.exists(user_id).await.unwrap_or(false)
    } else {
        false
    }
}

/// Check whether a set of users is online, returns a set of the online user IDs
pub async fn presence_filter_online(user_ids: &'_ [String]) -> HashSet<String> {
    // Ignore empty list immediately, to save time.
    let mut set = HashSet::new();
    if user_ids.is_empty() {
        return set;
    }

    // NOTE: at the point that we need mobile indicators
    // you can interpret the data here and return a new data
    // structure like HashMap<String /* id */, u8 /* flags */>

    // We need to handle a special case where only one is present
    // as for some reason or another, Redis does not like us sending
    // a list of just one ID to the server.
    if user_ids.len() == 1 {
        if presence_is_online(&user_ids[0]).await {
            set.insert(user_ids[0].to_string());
        }

        return set;
    }

    // Otherwise, go ahead as normal.
    if let Ok(mut conn) = get_connection().await {
        // Ok so, if this breaks, that means we've lost the Redis patch which adds SMISMEMBER
        // Currently it's patched in through a forked repository, investigate what happen to it
        let data: Vec<bool> = conn
            .sismember("online", user_ids)
            .await
            .expect("this shouldn't happen, please read this code! presence/mod.rs");
        if data.is_empty() {
            return set;
        }

        // We filter known values to figure out who is online.
        for i in 0..user_ids.len() {
            if data[i] {
                set.insert(user_ids[i].to_string());
            }
        }
    }

    set
}

/// Reset any stale presence data
pub async fn presence_clear_region(region_id: Option<&str>) {
    let region_id = region_id.unwrap_or(&*REGION_KEY);
    let mut conn = get_connection().await.expect("Redis connection");

    let sessions = __get_set_members_as_string(&mut conn, region_id).await;
    if !sessions.is_empty() {
        info!(
            "Cleaning up {} sessions, this may take a while...",
            sessions.len()
        );

        // Iterate and delete each session, this will
        // also send out any relevant events.
        for session in sessions {
            let parts = session.split(':').collect::<Vec<&str>>();
            if let (Some(user_id), Some(session_id)) = (parts.first(), parts.get(1)) {
                if let Ok(session_id) = session_id.parse() {
                    presence_delete_session_internal(user_id, session_id, true).await;
                }
            }
        }

        // Then clear the set in Redis.
        __delete_key(&mut conn, region_id).await;

        info!("Clean up complete.");
    }
}
