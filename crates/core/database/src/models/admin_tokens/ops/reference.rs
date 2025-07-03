use revolt_result::Result;

use crate::AdminToken;
use crate::ReferenceDb;

use super::AbstractAdminTokens;

#[async_trait]
impl AbstractAdminTokens for ReferenceDb {
    async fn admin_token_create(&self, token: AdminToken) -> Result<()> {
        let mut admin_tokens = self.admin_tokens.lock().await;
        if admin_tokens.contains_key(&token.id) {
            Err(create_database_error!("insert", "admin_tokens"))
        } else {
            admin_tokens.insert(token.id.to_string(), token.clone());
            Ok(())
        }
    }

    async fn admin_token_revoke(&self, token_id: &str) -> Result<()> {
        let mut admin_tokens = self.admin_tokens.lock().await;
        if admin_tokens.remove(token_id).is_some() {
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }

    async fn admin_token_authenticate(&self, token: &str) -> Result<AdminToken> {
        let admin_tokens = self.admin_tokens.lock().await;
        let result = admin_tokens
            .iter()
            .filter_map(|(_, t)| {
                if t.token == token {
                    Some(t.clone())
                } else {
                    None
                }
            })
            .next();

        Ok(result.ok_or_else(|| create_error!(NotFound))?)
    }

    async fn admin_token_fetch(&self, id: &str) -> Result<AdminToken> {
        let admin_tokens = self.admin_tokens.lock().await;
        let result = admin_tokens.iter().find(|tok| tok.0 == id);

        Ok(result
            .map(|t| t.1.clone())
            .ok_or_else(|| create_error!(NotFound))?)
    }
}
