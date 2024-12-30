use crate::models::user_white_list::UserWhiteList;
use crate::Result;

#[async_trait]
pub trait AbstractUserWhiteList: Sync + Send {
    async fn empty_white_list(&self) -> Result<()>;

    async fn fetch_white_list(&self, email: &str) -> Result<UserWhiteList>;

    async fn fetch_white_lists<'a>(&self) -> Result<Vec<UserWhiteList>>;

    async fn insert_white_list(&self, user_white_list: &UserWhiteList) -> Result<()>;
}
