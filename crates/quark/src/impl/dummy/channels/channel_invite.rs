use crate::models::Invite;
use crate::{AbstractChannelInvite, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractChannelInvite for DummyDb {
    async fn fetch_invite(&self, code: &str) -> Result<Invite> {
        Ok(Invite::Server {
            code: code.into(),
            server: "server".into(),
            creator: "creator".into(),
            channel: "channel".into(),
        })
    }

    async fn insert_invite(&self, invite: &Invite) -> Result<()> {
        info!("Insert {invite:?}");
        Ok(())
    }

    async fn delete_invite(&self, code: &str) -> Result<()> {
        info!("Delete {code}");
        Ok(())
    }

    async fn fetch_invites_for_server(&self, server: &str) -> Result<Vec<Invite>> {
        Ok(vec![self.fetch_invite(server).await.unwrap()])
    }
}
