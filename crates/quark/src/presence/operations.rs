use redis_kiss::{AsyncCommands, Conn};

use super::entry::PresenceEntry;

/// Set presence entry by given ID
pub async fn __set_key_presence_entry(conn: &mut Conn, id: &str, data: Vec<PresenceEntry>) {
    let _: Option<()> = conn.set(id, bincode::serialize(&data).unwrap()).await.ok();
}

/// Delete presence entry by given ID
pub async fn __delete_key_presence_entry(conn: &mut Conn, id: &str) {
    let _: Option<()> = conn.del(id).await.ok();
}

/// Get presence entry by given ID
pub async fn __get_key_presence_entry(conn: &mut Conn, id: &str) -> Option<Vec<PresenceEntry>> {
    conn.get::<_, Option<Vec<u8>>>(id)
        .await
        .unwrap()
        .map(|entry| bincode::deserialize(&entry[..]).unwrap())
}

/// Add to region session set
pub async fn __add_to_set_sessions(
    conn: &mut Conn,
    region_id: &str,
    user_id: &str,
    session_id: u8,
) {
    let _: Option<()> = conn
        .sadd(region_id, format!("{user_id}:{session_id}"))
        .await
        .ok();
}

/// Remove from region session set
pub async fn __remove_from_set_sessions(
    conn: &mut Conn,
    region_id: &str,
    user_id: &str,
    session_id: u8,
) {
    let _: Option<()> = conn
        .srem(region_id, format!("{user_id}:{session_id}"))
        .await
        .ok();
}

/// Get region session set as list
pub async fn __get_set_sessions(conn: &mut Conn, region_id: &str) -> Vec<String> {
    conn.smembers::<_, Vec<String>>(region_id).await.unwrap()
}

/// Delete region session set
pub async fn __delete_set_sessions(conn: &mut Conn, region_id: &str) {
    let _: () = conn.del(region_id).await.unwrap();
}
