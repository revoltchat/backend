use crate::models::Invite;
use crate::Result;

#[async_trait]
pub trait AbstractChannelInvite: Sync + Send {
    /// Fetch an invite by its id
    async fn fetch_invite(&self, code: &str) -> Result<Invite>;

    /// Insert a new invite into the database
    async fn insert_invite(&self, invite: &Invite) -> Result<()>;

    /// Delete an invite by its id
    async fn delete_invite(&self, code: &str) -> Result<()>;

    /// Fetch all invites for a server
    async fn fetch_invites_for_server(&self, server: &str) -> Result<Vec<Invite>>;
}
