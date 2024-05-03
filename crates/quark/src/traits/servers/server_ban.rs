use crate::models::server_member::MemberCompositeKey;
use crate::models::ServerBan;
use crate::Result;

#[async_trait]
pub trait AbstractServerBan: Sync + Send {
    /// Fetch a server ban by server and user id
    async fn fetch_ban(&self, server: &str, user: &str) -> Result<ServerBan>;

    /// Fetch all bans in a server
    async fn fetch_bans(&self, server: &str) -> Result<Vec<ServerBan>>;

    /// Insert new ban into database
    async fn insert_ban(&self, ban: &ServerBan) -> Result<()>;

    /// Delete a ban from the database
    async fn delete_ban(&self, id: &MemberCompositeKey) -> Result<()>;
}
