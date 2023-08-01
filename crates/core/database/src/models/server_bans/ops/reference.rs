use revolt_result::Result;

use crate::ReferenceDb;
use crate::{MemberCompositeKey, ServerBan};

use super::AbstractServerBans;

#[async_trait]
impl AbstractServerBans for ReferenceDb {
    /// Insert new ban into database
    async fn insert_ban(&self, ban: &ServerBan) -> Result<()> {
        let mut server_bans = self.server_bans.lock().await;
        if server_bans.contains_key(&ban.id) {
            Err(create_database_error!("insert", "ban"))
        } else {
            server_bans.insert(ban.id.clone(), ban.clone());
            Ok(())
        }
    }

    /// Fetch a server ban by server and user id
    async fn fetch_ban(&self, server_id: &str, user_id: &str) -> Result<ServerBan> {
        let server_bans = self.server_bans.lock().await;
        server_bans
            .get(&MemberCompositeKey {
                server: server_id.to_string(),
                user: user_id.to_string(),
            })
            .cloned()
            .ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch all bans in a server
    async fn fetch_bans(&self, server_id: &str) -> Result<Vec<ServerBan>> {
        let server_bans = self.server_bans.lock().await;
        Ok(server_bans
            .values()
            .filter(|member| member.id.server == server_id)
            .cloned()
            .collect())
    }

    /// Delete a ban from the database
    async fn delete_ban(&self, id: &MemberCompositeKey) -> Result<()> {
        let mut server_bans = self.server_bans.lock().await;
        if server_bans.remove(id).is_some() {
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }
}
