use iso8601_timestamp::Timestamp;
use revolt_result::Result;

use crate::ReferenceDb;
use crate::{AdminCaseComment, PartialAdminCaseComment};

use super::AbstractAdminCaseComments;

#[async_trait]
impl AbstractAdminCaseComments for ReferenceDb {
    async fn admin_case_comment_insert(&self, comment: AdminCaseComment) -> Result<()> {
        let mut admin_case_comments = self.admin_case_comments.lock().await;
        if admin_case_comments.contains_key(&comment.id) {
            Err(create_database_error!("insert", "admin_case_comments"))
        } else {
            admin_case_comments.insert(comment.id.to_string(), comment.clone());
            Ok(())
        }
    }

    async fn admin_case_comment_update(
        &self,
        comment_id: &str,
        partial: &PartialAdminCaseComment,
    ) -> Result<()> {
        let mut admin_case_comments = self.admin_case_comments.lock().await;
        if let Some(comment) = admin_case_comments.get_mut(comment_id) {
            comment.apply_options(partial.clone());
            comment.edited_at = Some(Timestamp::now_utc().format().to_string());
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }

    async fn admin_case_comment_fetch(&self, case_id: &str) -> Result<Vec<AdminCaseComment>> {
        let admin_case_comments = self.admin_case_comments.lock().await;
        Ok(admin_case_comments
            .iter()
            .filter_map(|(_, c)| {
                if c.case_id == case_id {
                    Some(c.clone())
                } else {
                    None
                }
            })
            .collect())
    }
}
