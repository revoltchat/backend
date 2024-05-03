use crate::models::server_member::{FieldsMember, Member, MemberCompositeKey, PartialMember};
use crate::Result;

#[async_trait]
pub trait AbstractServerMember: Sync + Send {
    /// Fetch a server member by their id
    async fn fetch_member(&self, server: &str, user: &str) -> Result<Member>;

    /// Insert a new server member into the database
    async fn insert_member(&self, member: &Member) -> Result<()>;

    /// Update information for a server member
    async fn update_member(
        &self,
        id: &MemberCompositeKey,
        member: &PartialMember,
        remove: Vec<FieldsMember>,
    ) -> Result<()>;

    /// Delete a server member by their id
    async fn delete_member(&self, id: &MemberCompositeKey) -> Result<()>;

    /// Fetch all members in a server
    async fn fetch_all_members<'a>(&self, server: &str) -> Result<Vec<Member>>;

    /// Fetch all memberships for a user
    async fn fetch_all_memberships<'a>(&self, user: &str) -> Result<Vec<Member>>;

    /// Fetch multiple members by their ids
    async fn fetch_members<'a>(&self, server: &str, ids: &'a [String]) -> Result<Vec<Member>>;

    /// Fetch member count of a server
    async fn fetch_member_count(&self, server: &str) -> Result<usize>;

    /// Fetch server count of a user
    async fn fetch_server_count(&self, user: &str) -> Result<usize>;
}
