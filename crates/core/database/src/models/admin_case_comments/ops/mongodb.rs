use revolt_result::Result;

use crate::MongoDb;
use crate::{AdminCaseComment, PartialAdminCaseComment};

use super::AbstractAdminCaseComments;

static COL: &str = "admin_case_comments";

#[async_trait]
impl AbstractAdminCaseComments for MongoDb {
    async fn admin_case_comment_insert(&self, comment: AdminCaseComment) -> Result<()> {
        query!(self, insert_one, COL, comment).map(|_| ())
    }

    async fn admin_case_comment_update(
        &self,
        comment_id: &str,
        partial: &PartialAdminCaseComment,
    ) -> Result<()> {
        query!(
            self,
            update_one_by_id,
            COL,
            comment_id,
            partial,
            vec![],
            None
        )
        .map(|_| ())
    }

    async fn admin_case_comment_fetch(&self, case_id: &str) -> Result<Vec<AdminCaseComment>> {
        query!(self, find, COL, doc! {"case_id": case_id})
    }
}
