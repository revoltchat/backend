use crate::models::Invite;
use crate::{AbstractChannelInvite, Result};

use super::super::MongoDb;

static COL: &str = "channel_invites";

#[async_trait]
impl AbstractChannelInvite for MongoDb {
    async fn fetch_invite(&self, code: &str) -> Result<Invite> {
        self.find_one_by_id(COL, code).await
    }

    async fn insert_invite(&self, invite: &Invite) -> Result<()> {
        self.insert_one(COL, invite).await.map(|_| ())
    }

    async fn delete_invite(&self, code: &str) -> Result<()> {
        self.delete_one_by_id(COL, code).await.map(|_| ())
    }

    async fn fetch_invites_for_server(&self, server: &str) -> Result<Vec<Invite>> {
        self.find(
            COL,
            doc! {
                "server": server
            },
        )
        .await
    }
}
