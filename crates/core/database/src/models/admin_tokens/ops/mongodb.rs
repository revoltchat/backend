use revolt_result::Result;

use crate::AdminToken;
use crate::MongoDb;

use super::AbstractAdminTokens;

static COL: &str = "admin_tokens";

#[async_trait]
impl AbstractAdminTokens for MongoDb {
    async fn admin_token_create(&self, token: AdminToken) -> Result<()> {
        query!(self, insert_one, COL, token).map(|_| ())
    }

    async fn admin_token_revoke(&self, token_id: &str) -> Result<()> {
        query!(self, delete_one_by_id, COL, token_id).map(|k| {
            if k.deleted_count > 0 {
                Ok(())
            } else {
                Err(create_error!(NotFound))
            }
        })?
    }

    async fn admin_token_authenticate(&self, token: &str) -> Result<AdminToken> {
        query!(self, find_one, COL, doc! {"token": token})?
            .ok_or_else(|| create_error!(InvalidCredentials))
    }

    async fn admin_token_fetch(&self, id: &str) -> Result<AdminToken> {
        query!(self, find_one_by_id, COL, id)?.ok_or_else(|| create_error!(NotFound))
    }
}
