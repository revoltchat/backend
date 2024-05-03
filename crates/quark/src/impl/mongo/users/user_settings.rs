use bson::{to_bson, Document};
use mongodb::options::{FindOneOptions, UpdateOptions};

use crate::models::UserSettings;
use crate::{AbstractUserSettings, Error, Result};

use super::super::MongoDb;

static COL: &str = "user_settings";

#[async_trait]
impl AbstractUserSettings for MongoDb {
    async fn fetch_user_settings(&'_ self, id: &str, filter: &'_ [String]) -> Result<UserSettings> {
        let mut projection = doc! {
            "_id": 0,
        };

        for key in filter {
            projection.insert(key, 1);
        }

        self.find_one_with_options(
            COL,
            doc! {
                "_id": id
            },
            FindOneOptions::builder().projection(projection).build(),
        )
        .await
    }

    async fn set_user_settings(&self, id: &str, settings: &UserSettings) -> Result<()> {
        let mut set = doc! {};
        for (key, data) in settings {
            set.insert(
                key,
                vec![to_bson(&data.0).unwrap(), to_bson(&data.1).unwrap()],
            );
        }

        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id": id
                },
                doc! {
                    "$set": set
                },
                UpdateOptions::builder().upsert(true).build(),
            )
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "user_settings",
            })
    }

    async fn delete_user_settings(&self, id: &str) -> Result<()> {
        self.delete_one_by_id(COL, id).await.map(|_| ())
    }
}
