use revolt_result::Result;

use crate::Invite;

#[cfg(feature = "mongodb")]
mod mongodb;
mod reference;

#[async_trait]
pub trait AbstractChannelInvites: Sync + Send {
    /// Insert a new invite into the database
    async fn insert_invite(&self, invite: &Invite) -> Result<()>;

    /// Fetch an invite by its id
    async fn fetch_invite(&self, code: &str) -> Result<Invite>;

    /// Fetch all invites for a server
    async fn fetch_invites_for_server(&self, server_id: &str) -> Result<Vec<Invite>>;

    /// Delete an invite by its id
    async fn delete_invite(&self, code: &str) -> Result<()>;

    /// Count one use against an invite
    async fn increment_invite_uses(&self, code: &str) -> Result<()>;

    /// Remove an invite code from the AUTHIFIER invite store.
    ///
    /// This reaches into another crate's collection, which is unusual enough to
    /// justify itself. Registration is gated by authifier, not by us: its
    /// `create_account` looks the code up with `find_invite` and - measured
    /// against authifier 1.0.16 - **never checks the `used` flag it sets**. So
    /// a code marked used still registers accounts forever, and the crate
    /// exposes no `delete_invite` of its own.
    ///
    /// Removing the mirror row is therefore the only way to make a use limit
    /// hold for ACCOUNT CREATION as well as for joining, short of forking the
    /// crate. Our own code is what writes that row in the first place, in
    /// `invite_create`, so we are removing something we put there.
    async fn delete_authifier_invite_mirror(&self, code: &str) -> Result<()>;
}
