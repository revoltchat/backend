use crate::models::server_member::{FieldsMember, Member, MemberCompositeKey, PartialMember};
use crate::{AbstractServerMember, Result};

use super::super::DummyDb;

use iso8601_timestamp::Timestamp;

#[async_trait]
impl AbstractServerMember for DummyDb {
    async fn fetch_member(&self, server: &str, user: &str) -> Result<Member> {
        Ok(Member {
            id: MemberCompositeKey {
                server: server.into(),
                user: user.into(),
            },
            joined_at: Timestamp::now_utc(),
            nickname: None,
            avatar: None,
            roles: vec![],
            timeout: None,
        })
    }

    async fn insert_member(&self, member: &Member) -> Result<()> {
        info!("Create {member:?}");
        Ok(())
    }

    async fn update_member(
        &self,
        id: &MemberCompositeKey,
        member: &PartialMember,
        remove: Vec<FieldsMember>,
    ) -> Result<()> {
        info!("Update {id:?} with {member:?} and remove {remove:?}");
        Ok(())
    }

    async fn delete_member(&self, id: &MemberCompositeKey) -> Result<()> {
        info!("Delete {id:?}");
        Ok(())
    }

    async fn fetch_all_members<'a>(&self, server: &str) -> Result<Vec<Member>> {
        Ok(vec![self.fetch_member(server, "member").await.unwrap()])
    }

    async fn fetch_all_memberships<'a>(&self, user: &str) -> Result<Vec<Member>> {
        Ok(vec![self.fetch_member("server", user).await.unwrap()])
    }

    async fn fetch_members<'a>(&self, server: &str, _ids: &'a [String]) -> Result<Vec<Member>> {
        Ok(vec![self.fetch_member(server, "member").await.unwrap()])
    }

    async fn fetch_member_count(&self, _server: &str) -> Result<usize> {
        Ok(100)
    }

    async fn fetch_server_count(&self, _user: &str) -> Result<usize> {
        Ok(5)
    }
}
