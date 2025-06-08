mod mongodb;
mod reference;

use revolt_result::Result;

use crate::{models::admin_case_comments::AdminCaseComment, PartialAdminCaseComment};

#[async_trait]
pub trait AbstractAdminCaseComments: Sync + Send {
    async fn admin_case_comment_insert(&self, comment: AdminCaseComment) -> Result<()>;

    async fn admin_case_comment_update(
        &self,
        comment_id: &str,
        partial: &PartialAdminCaseComment,
    ) -> Result<()>;

    async fn admin_case_comment_fetch(&self, case_id: &str) -> Result<Vec<AdminCaseComment>>;
}
