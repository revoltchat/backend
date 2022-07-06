use crate::models::attachment::File;
use crate::Result;

#[async_trait]
pub trait AbstractAttachment: Sync + Send {
    async fn find_and_use_attachment(
        &self,
        id: &str,
        tag: &str,
        parent_type: &str,
        parent_id: &str,
    ) -> Result<File>;
    async fn insert_attachment(&self, attachment: &File) -> Result<()>;
    async fn mark_attachment_as_reported(&self, id: &str) -> Result<()>;
    async fn mark_attachment_as_deleted(&self, id: &str) -> Result<()>;
    async fn mark_attachments_as_deleted(&self, ids: &[String]) -> Result<()>;
}
