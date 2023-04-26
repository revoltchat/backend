use revolt_result::Result;

use crate::{FieldsMember, Member, MemberCompositeKey, PartialMember};

mod mongodb;
mod reference;

#[async_trait]
pub trait AbstractServerMembers: Sync + Send {
    /// Insert a new server member into the database
    async fn insert_member(&self, member: &Member) -> Result<()>;

    /// Fetch a server member by their id
    async fn fetch_member(&self, server_id: &str, user_id: &str) -> Result<Member>;

    /// Fetch all members in a server
    async fn fetch_all_members<'a>(&self, server_id: &str) -> Result<Vec<Member>>;

    /// Fetch all memberships for a user
    async fn fetch_all_memberships<'a>(&self, user_id: &str) -> Result<Vec<Member>>;

    /// Fetch multiple members by their ids
    async fn fetch_members<'a>(&self, server_id: &str, ids: &'a [String]) -> Result<Vec<Member>>;

    /// Fetch member count of a server
    async fn fetch_member_count(&self, server_id: &str) -> Result<usize>;

    /// Fetch server count of a user
    async fn fetch_server_count(&self, user_id: &str) -> Result<usize>;

    /// Update information for a server member
    async fn update_member(
        &self,
        id: &MemberCompositeKey,
        partial: &PartialMember,
        remove: Vec<FieldsMember>,
    ) -> Result<()>;

    /// Delete a server member by their id
    async fn delete_member(&self, id: &MemberCompositeKey) -> Result<()>;
}
