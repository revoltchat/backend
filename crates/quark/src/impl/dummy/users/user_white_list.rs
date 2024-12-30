use crate::models::user_white_list::UserWhiteList;
use crate::{AbstractUserWhiteList, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractUserWhiteList for DummyDb {
    async fn empty_white_list(&self) -> Result<()> {
        Ok(())
    }
    async fn fetch_white_list(&self, email: &str) -> Result<UserWhiteList> {
        Ok(UserWhiteList {
            email: email.into(),
            name: todo!(),
            phone_number: todo!(),
        })
    }
    async fn fetch_white_lists<'a>(&self) -> Result<Vec<UserWhiteList>> {
        Ok(vec![self.fetch_white_list("email").await.unwrap()])
    }
    async fn insert_white_list(&self, white_list: &UserWhiteList) -> Result<()> {
        info!("Insert {white_list:?}");
        Ok(())
    }
}
