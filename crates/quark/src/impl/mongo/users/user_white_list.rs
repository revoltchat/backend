use crate::models::user_white_list::UserWhiteList;
use crate::{AbstractUserWhiteList, Result};

use super::super::MongoDb;

static COL: &str = "user_white_list";

#[async_trait]
impl AbstractUserWhiteList for MongoDb {
    async fn empty_white_list(&self) -> Result<()> {
        self.col::<UserWhiteList>("user_white_list")
            .delete_many(doc! {}, None)
            .await;
        Ok(())
    }

    async fn fetch_white_list(&self, id: &str) -> Result<UserWhiteList> {
        self.find_one(
            COL,
            doc! {
                "email": id
            },
        )
        .await
    }

    async fn fetch_white_lists<'a>(&self) -> Result<Vec<UserWhiteList>> {
        self.find(COL, doc! {}).await
    }

    async fn insert_white_list(&self, user_white_list: &UserWhiteList) -> Result<()> {
        self.insert_one(COL, user_white_list).await.map(|_| ())
    }
}
