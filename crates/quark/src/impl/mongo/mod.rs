use std::ops::Deref;

use bson::{to_document, Document};
use futures::StreamExt;
use mongodb::{
    options::{FindOneOptions, FindOptions},
    results::{DeleteResult, InsertOneResult, UpdateResult},
};
use rocket::serde::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{util::manipulation::prefix_keys, AbstractDatabase, Error, Result};

pub mod admin {
    pub mod stats;
}

pub mod media {
    pub mod attachment;
    pub mod emoji;
}

pub mod channels {
    pub mod channel;
    pub mod channel_invite;
    pub mod channel_unread;
    pub mod message;
}

pub mod servers {
    pub mod server;
    pub mod server_ban;
    pub mod server_member;
}

pub mod users {
    pub mod bot;
    pub mod user;
    pub mod user_settings;
}

pub mod safety {
    pub mod report;
    pub mod snapshot;
}

#[derive(Debug, Clone)]
pub struct MongoDb(pub mongodb::Client);

impl AbstractDatabase for MongoDb {}

impl Deref for MongoDb {
    type Target = mongodb::Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MongoDb {
    pub fn db(&self) -> mongodb::Database {
        self.database("revolt")
    }

    pub fn col<T>(&self, collection: &str) -> mongodb::Collection<T> {
        self.db().collection(collection)
    }

    async fn insert_one<T: Serialize>(
        &self,
        collection: &'static str,
        document: T,
    ) -> Result<InsertOneResult> {
        self.col::<T>(collection)
            .insert_one(document, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "insert_one",
                with: collection,
            })
    }

    async fn find_with_options<O, T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        projection: Document,
        options: O,
    ) -> Result<Vec<T>>
    where
        O: Into<Option<FindOptions>>,
    {
        let result = self.col::<T>(collection).find(projection, options).await;
        Ok(if cfg!(debug_assertions) {
            result.unwrap()
        } else {
            result.map_err(|_| Error::DatabaseError {
                operation: "find",
                with: collection,
            })?
        }
        .filter_map(|s| async {
            if cfg!(debug_assertions) {
                // Hard fail on invalid documents
                Some(s.unwrap())
            } else {
                s.ok()
            }
        })
        .collect::<Vec<T>>()
        .await)
    }

    async fn find<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        projection: Document,
    ) -> Result<Vec<T>> {
        self.find_with_options(collection, projection, None).await
    }

    async fn find_one_with_options<O, T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        projection: Document,
        options: O,
    ) -> Result<T>
    where
        O: Into<Option<FindOneOptions>>,
    {
        self.col::<T>(collection)
            .find_one(projection, options)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: collection,
            })?
            .ok_or(Error::NotFound)
    }

    async fn find_one<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        projection: Document,
    ) -> Result<T> {
        self.find_one_with_options(collection, projection, None)
            .await
    }

    async fn find_one_by_id<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        id: &str,
    ) -> Result<T> {
        self.find_one(
            collection,
            doc! {
                "_id": id
            },
        )
        .await
    }

    async fn update_one<P, T: Serialize>(
        &self,
        collection: &'static str,
        projection: Document,
        partial: T,
        remove: Vec<&dyn IntoDocumentPath>,
        prefix: P,
    ) -> Result<UpdateResult>
    where
        P: Into<Option<String>>,
    {
        let prefix = prefix.into();

        let mut unset = doc! {};
        for field in remove {
            if let Some(path) = field.as_path() {
                if let Some(prefix) = &prefix {
                    unset.insert(prefix.to_owned() + path, 1_i32);
                } else {
                    unset.insert(path, 1_i32);
                }
            }
        }

        let query = doc! {
            "$unset": unset,
            "$set": if let Some(prefix) = &prefix {
                to_document(&prefix_keys(&partial, prefix))
            } else {
                to_document(&partial)
            }.map_err(|_| Error::DatabaseError {
                operation: "to_document",
                with: collection
            })?
        };

        self.col::<Document>(collection)
            .update_one(projection, query, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: collection,
            })
    }

    async fn update_one_by_id<P, T: Serialize>(
        &self,
        collection: &'static str,
        id: &str,
        partial: T,
        remove: Vec<&dyn IntoDocumentPath>,
        prefix: P,
    ) -> Result<UpdateResult>
    where
        P: Into<Option<String>>,
    {
        self.update_one(
            collection,
            doc! {
                "_id": id
            },
            partial,
            remove,
            prefix,
        )
        .await
    }

    async fn delete_one(
        &self,
        collection: &'static str,
        projection: Document,
    ) -> Result<DeleteResult> {
        self.col::<Document>(collection)
            .delete_one(projection, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_one",
                with: collection,
            })
    }

    async fn delete_one_by_id(&self, collection: &'static str, id: &str) -> Result<DeleteResult> {
        self.delete_one(
            collection,
            doc! {
                "_id": id
            },
        )
        .await
    }
}

#[derive(Deserialize)]
pub struct DocumentId {
    #[serde(rename = "_id")]
    pub id: String,
}

pub trait IntoDocumentPath: Send + Sync {
    fn as_path(&self) -> Option<&'static str>;
}
