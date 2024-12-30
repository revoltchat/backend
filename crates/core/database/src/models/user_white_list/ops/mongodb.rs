use bson::{to_document, Document};
use futures::StreamExt;
use mongodb::options::Collation;
use mongodb::options::CollationStrength;
use mongodb::options::FindOneOptions;
use revolt_result::Result;

use crate::MongoDb;
use crate::UserWhiteList;

use super::AbstractUserWhiteList;

static COL: &str = "user_white_list";

#[async_trait]
impl AbstractUserWhiteList for MongoDb {
    async fn insert_user_white_list(&self, user_white_list: &UserWhiteList) -> Result<()> {
        query!(self, insert_one, COL, &user_white_list).map(|_| ())
    }

    async fn fetch_user_white_list(&self, email: &str) -> Result<UserWhiteList> {
        query!(
            self,
            find_one_with_options,
            COL,
            doc! {
                "email": email
            },
            FindOneOptions::builder()
                .collation(
                    Collation::builder()
                        .locale("en")
                        .strength(CollationStrength::Secondary)
                        .build(),
                )
                .build()
        )?
        .ok_or_else(|| create_error!(NotFound))
    }

    async fn fetch_user_white_lists<'a>(&self) -> Result<Vec<UserWhiteList>> {
        Ok(self
            .col::<UserWhiteList>(COL)
            .find(doc! {}, None)
            .await
            .map_err(|_| create_database_error!("find", "user_white_list"))?
            .filter_map(|s| async {
                if cfg!(debug_assertions) {
                    Some(s.unwrap())
                } else {
                    s.ok()
                }
            })
            .collect()
            .await)
    }
}
