use bson::Document;

use crate::models::attachment::File;
use crate::{AbstractAttachment, Error, Result};

use super::super::MongoDb;

static COL: &str = "attachments";

impl MongoDb {
    pub async fn delete_many_attachments(&self, projection: Document) -> Result<()> {
        self.col::<Document>(COL)
            .update_many(
                projection,
                doc! {
                    "$set": {
                        "deleted": true
                    }
                },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "update_many",
                with: "attachment",
            })
    }
}

#[async_trait]
impl AbstractAttachment for MongoDb {
    async fn find_and_use_attachment(
        &self,
        attachment_id: &str,
        tag: &str,
        parent_type: &str,
        parent_id: &str,
    ) -> Result<File> {
        let key = format!("{}_id", parent_type);
        match self
            .find_one::<File>(
                COL,
                doc! {
                    "_id": attachment_id,
                    "tag": tag,
                    &key: {
                        "$exists": false
                    }
                },
            )
            .await
        {
            Ok(file) => {
                self.col::<Document>(COL)
                    .update_one(
                        doc! {
                            "_id": &file.id
                        },
                        doc! {
                            "$set": {
                                key: parent_id
                            }
                        },
                        None,
                    )
                    .await
                    .map_err(|_| Error::DatabaseError {
                        operation: "update_one",
                        with: "attachment",
                    })?;

                Ok(file)
            }
            Err(Error::NotFound) => Err(Error::UnknownAttachment),
            Err(error) => Err(error),
        }
    }

    async fn insert_attachment(&self, attachment: &File) -> Result<()> {
        self.insert_one(COL, attachment).await.map(|_| ())
    }

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
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "attachment",
            })
    }

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
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "attachment",
            })
    }

    async fn mark_attachments_as_deleted(&self, ids: &[String]) -> Result<()> {
        self.col::<Document>(COL)
            .update_many(
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
            .map_err(|_| Error::DatabaseError {
                operation: "update",
                with: "attachments",
            })
    }
}
