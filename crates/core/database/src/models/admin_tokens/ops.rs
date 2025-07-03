mod mongodb;
mod reference;
use revolt_result::Result;

use crate::models::admin_tokens::AdminToken;

#[async_trait]
pub trait AbstractAdminTokens: Sync + Send {
    async fn admin_token_create(&self, token: AdminToken) -> Result<()>;

    async fn admin_token_revoke(&self, token_id: &str) -> Result<()>;

    async fn admin_token_authenticate(&self, token: &str) -> Result<AdminToken>;

    async fn admin_token_fetch(&self, id: &str) -> Result<AdminToken>;
}
