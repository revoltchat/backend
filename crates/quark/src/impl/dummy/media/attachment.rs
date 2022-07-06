use crate::models::attachment::File;
use crate::{AbstractAttachment, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractAttachment for DummyDb {
    async fn find_and_use_attachment(
        &self,
        attachment_id: &str,
        tag: &str,
        _parent_type: &str,
        parent_id: &str,
    ) -> Result<File> {
        Ok(File {
            id: attachment_id.into(),
            tag: tag.into(),
            filename: "file.txt".into(),
            content_type: "plain/text".into(),
            size: 100,

            object_id: Some(parent_id.into()),

            ..Default::default()
        })
    }

    async fn insert_attachment(&self, attachment: &File) -> Result<()> {
        info!("Insert {attachment:?}");
        Ok(())
    }

    async fn mark_attachment_as_reported(&self, id: &str) -> Result<()> {
        info!("Marked {id} as reported");
        Ok(())
    }

    async fn mark_attachment_as_deleted(&self, id: &str) -> Result<()> {
        info!("Marked {id} as deleted");
        Ok(())
    }

    async fn mark_attachments_as_deleted(&self, ids: &[String]) -> Result<()> {
        info!("Marked {ids:?} as deleted");
        Ok(())
    }
}
