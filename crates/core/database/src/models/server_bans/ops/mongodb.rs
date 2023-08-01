use revolt_result::Result;

use crate::MongoDb;
use crate::{MemberCompositeKey, ServerBan};

use super::AbstractServerBans;

static COL: &str = "server_bans";

#[async_trait]
impl AbstractServerBans for MongoDb {
    /// Insert new ban into database
    async fn insert_ban(&self, ban: &ServerBan) -> Result<()> {
        query!(self, insert_one, COL, &ban).map(|_| ())
    }

    /// Fetch a server ban by server and user id
    async fn fetch_ban(&self, server_id: &str, user_id: &str) -> Result<ServerBan> {
        query!(
            self,
            find_one,
            COL,
            doc! {
                "_id.server": server_id,
                "_id.user": user_id
            }
        )?
        .ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch all bans in a server
    async fn fetch_bans(&self, server_id: &str) -> Result<Vec<ServerBan>> {
        query!(
            self,
            find,
            COL,
            doc! {
                "_id.server": server_id
            }
        )
    }

    /// Delete a ban from the database
    async fn delete_ban(&self, id: &MemberCompositeKey) -> Result<()> {
        query!(
            self,
            delete_one,
            COL,
            doc! {
                "_id.server": &id.server,
                "_id.user": &id.user
            }
        )
        .map(|_| ())
    }
}
