use std::collections::HashMap;

use bson::{from_document, Document};
use futures::StreamExt;

use crate::{
    models::stats::{Index, Stats},
    AbstractStats, Error, Result,
};

use super::super::MongoDb;

#[async_trait]
impl AbstractStats for MongoDb {
    async fn generate_stats(&self) -> Result<Stats> {
        let mut indices = HashMap::new();
        let mut coll_stats = HashMap::new();

        let collection_names =
            self.db()
                .list_collection_names(None)
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "list_collection_names",
                    with: "database",
                })?;

        for collection in collection_names {
            indices.insert(
                collection.to_string(),
                self.col::<Document>(&collection)
                    .aggregate(
                        vec![doc! {
                           "$indexStats": { }
                        }],
                        None,
                    )
                    .await
                    .map_err(|_| Error::DatabaseError {
                        operation: "aggregate",
                        with: "col",
                    })?
                    .filter_map(|s| async { s.ok() })
                    .collect::<Vec<Document>>()
                    .await
                    .into_iter()
                    .filter_map(|doc| from_document(doc).ok())
                    .collect::<Vec<Index>>(),
            );

            coll_stats.insert(
                collection.to_string(),
                self.col::<Document>(&collection)
                    .aggregate(
                        vec![doc! {
                           "$collStats": {
                                "latencyStats": {
                                    "histograms": true
                                },
                                "storageStats": {},
                                "count": {},
                                "queryExecStats": {}
                            }
                        }],
                        None,
                    )
                    .await
                    .map_err(|_| Error::DatabaseError {
                        operation: "aggregate",
                        with: "col",
                    })?
                    .filter_map(|s| async { s.ok() })
                    .collect::<Vec<Document>>()
                    .await
                    .into_iter()
                    .filter_map(|doc| from_document(doc).ok())
                    .next()
                    .ok_or(Error::DatabaseError {
                        operation: "next aggregation",
                        with: "col",
                    })?,
            );
        }

        Ok(Stats {
            indices,
            coll_stats,
        })
    }
}
