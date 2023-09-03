use bson::Document;
use revolt_result::Result;

use crate::File;
use crate::MongoDb;

use super::AbstractAttachments;

static COL: &str = "attachments";

#[async_trait]
impl AbstractAttachments for MongoDb {
    /// Insert attachment into database.
    async fn insert_attachment(&self, attachment: &File) -> Result<()> {
        query!(self, insert_one, COL, &attachment).map(|_| ())
    }

    /// Find an attachment by its details and mark it as used by a given parent.
    async fn find_and_use_attachment(
        &self,
        id: &str,
        tag: &str,
        parent_type: &str,
        parent_id: &str,
    ) -> Result<File> {
        let key = format!("{parent_type}_id");
        let file = query!(
            self,
            find_one,
            COL,
            doc! {
                "_id": id,
                "tag": tag,
                &key: {
                    "$exists": false
                }
            }
        )?
        .ok_or_else(|| create_error!(NotFound))?;

        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id": id
                },
                doc! {
                    "$set": {
                        key: parent_id
                    }
                },
                None,
            )
            .await
            .map_err(|_| create_database_error!("update_one", COL))?;

        Ok(file)
    }

    /// Mark an attachment as having been reported.
    async fn mark_attachment_as_reported(&self, id: &str) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id": id
                },
                doc! {
                    "$set": {
                        "reported": true
                    }
                },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| create_database_error!("update_one", COL))
    }

    /// Mark an attachment as having been deleted.
    async fn mark_attachment_as_deleted(&self, id: &str) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id": id
                },
                doc! {
                    "$set": {
                        "deleted": true
                    }
                },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| create_database_error!("update_one", COL))
    }

    /// Mark multiple attachments as having been deleted.
    async fn mark_attachments_as_deleted(&self, ids: &[String]) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id": {
                        "$in": ids
                    }
                },
                doc! {
                    "$set": {
                        "deleted": true
                    }
                },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| create_database_error!("update_one", COL))
    }
}
