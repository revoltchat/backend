use revolt_result::Result;

use crate::AdminAuditItem;
use crate::ReferenceDb;

use super::AbstractAdminAudits;

#[async_trait]
impl AbstractAdminAudits for ReferenceDb {
    async fn admin_audit_insert(&self, audit: AdminAuditItem) -> Result<()> {
        let mut admin_audits = self.admin_audits.lock().await;
        if admin_audits.contains_key(&audit.id) {
            Err(create_database_error!("insert", "admin_audits"))
        } else {
            admin_audits.insert(audit.id.to_string(), audit.clone());
            Ok(())
        }
    }

    async fn admin_audit_fetch(
        &self,
        before_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<AdminAuditItem>> {
        let admin_audits = self.admin_audits.lock().await;
        if let Some(before) = before_id {
            Ok(admin_audits
                .iter()
                .rev()
                .skip_while(|(id, _)| id.as_str() <= before)
                .by_ref()
                .take(limit as usize)
                .map(|(_, item)| item.clone())
                .collect())
        } else {
            Ok(admin_audits
                .iter()
                .rev()
                .by_ref()
                .take(limit as usize)
                .map(|(_, item)| item.clone())
                .collect())
        }
    }
}
