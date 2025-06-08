mod mongodb;
mod reference;

use revolt_result::Result;

use crate::models::admin_audits::AdminAuditItem;

#[async_trait]
pub trait AbstractAdminAudits: Sync + Send {
    async fn admin_audit_insert(&self, audit: AdminAuditItem) -> Result<()>;

    async fn admin_audit_fetch(
        &self,
        before_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<AdminAuditItem>>;
}
