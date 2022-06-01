use crate::models::server_member::MemberCompositeKey;
use crate::models::ServerBan;
use crate::{AbstractServerBan, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractServerBan for DummyDb {
    async fn fetch_ban(&self, server: &str, user: &str) -> Result<ServerBan> {
        Ok(ServerBan {
            id: MemberCompositeKey {
                server: server.into(),
                user: user.into(),
            },
            reason: Some("ban reason".into()),
        })
    }

    async fn fetch_bans(&self, server: &str) -> Result<Vec<ServerBan>> {
        Ok(vec![self.fetch_ban(server, "user").await.unwrap()])
    }

    async fn insert_ban(&self, ban: &ServerBan) -> Result<()> {
        info!("Insert {ban:?}");
        Ok(())
    }

    async fn delete_ban(&self, id: &MemberCompositeKey) -> Result<()> {
        info!("Delete {id:?}");
        Ok(())
    }
}
