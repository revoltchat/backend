use revolt_result::Result;

use crate::UserWhiteList;

mod mongodb;
mod reference;

#[async_trait]
pub trait AbstractUserWhiteList: Sync + Send {
    async fn insert_user_white_list(&self, user_white_list: &UserWhiteList) -> Result<()>;

    async fn fetch_user_white_list(&self, email: &str) -> Result<UserWhiteList>;

    async fn fetch_user_white_lists<'a>(&self) -> Result<Vec<UserWhiteList>>;
}
