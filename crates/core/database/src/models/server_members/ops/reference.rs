use revolt_result::Result;

use crate::ReferenceDb;
use crate::{FieldsMember, Member, MemberCompositeKey, PartialMember};

use super::AbstractServerMembers;

#[async_trait]
impl AbstractServerMembers for ReferenceDb {
    /// Insert a new server member into the database
    async fn insert_member(&self, member: &Member) -> Result<()> {
        let mut server_members = self.server_members.lock().await;
        if server_members.contains_key(&member.id) {
            Err(create_database_error!("insert", "member"))
        } else {
            server_members.insert(member.id.clone(), member.clone());
            Ok(())
        }
    }

    /// Fetch a server member by their id
    async fn fetch_member(&self, server_id: &str, user_id: &str) -> Result<Member> {
        let server_members = self.server_members.lock().await;
        server_members
            .get(&MemberCompositeKey {
                server: server_id.to_string(),
                user: user_id.to_string(),
            })
            .cloned()
            .ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch all members in a server
    async fn fetch_all_members<'a>(&self, server_id: &str) -> Result<Vec<Member>> {
        let server_members = self.server_members.lock().await;
        Ok(server_members
            .values()
            .filter(|member| member.id.server == server_id)
            .cloned()
            .collect())
    }

    /// Fetch all memberships for a user
    async fn fetch_all_memberships<'a>(&self, user_id: &str) -> Result<Vec<Member>> {
        let server_members = self.server_members.lock().await;
        Ok(server_members
            .values()
            .filter(|member| member.id.user == user_id)
            .cloned()
            .collect())
    }

    /// Fetch multiple members by their ids
    async fn fetch_members<'a>(&self, server_id: &str, ids: &'a [String]) -> Result<Vec<Member>> {
        let server_members = self.server_members.lock().await;
        ids.iter()
            .map(|id| {
                server_members
                    .get(&MemberCompositeKey {
                        server: server_id.to_string(),
                        user: id.to_string(),
                    })
                    .cloned()
                    .ok_or_else(|| create_error!(NotFound))
            })
            .collect()
    }

    /// Fetch member count of a server
    async fn fetch_member_count(&self, server_id: &str) -> Result<usize> {
        let server_members = self.server_members.lock().await;
        Ok(server_members
            .values()
            .filter(|member| member.id.server == server_id)
            .count())
    }

    /// Fetch server count of a user
    async fn fetch_server_count(&self, user_id: &str) -> Result<usize> {
        let server_members = self.server_members.lock().await;
        Ok(server_members
            .values()
            .filter(|member| member.id.user == user_id)
            .count())
    }

    /// Update information for a server member
    async fn update_member(
        &self,
        id: &MemberCompositeKey,
        partial: &PartialMember,
        remove: Vec<FieldsMember>,
    ) -> Result<()> {
        let mut server_members = self.server_members.lock().await;
        if let Some(member) = server_members.get_mut(id) {
            for field in remove {
                #[allow(clippy::disallowed_methods)]
                member.remove_field(&field);
            }

            member.apply_options(partial.clone());
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }

    /// Delete a server member by their id
    async fn delete_member(&self, id: &MemberCompositeKey) -> Result<()> {
        let mut server_members = self.server_members.lock().await;
        if server_members.remove(id).is_some() {
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }
}
