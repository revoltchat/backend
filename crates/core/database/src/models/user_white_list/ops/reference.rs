use revolt_result::Result;

use crate::ReferenceDb;
use crate::UserWhiteList;

use super::AbstractUserWhiteList;

#[async_trait]
impl AbstractUserWhiteList for ReferenceDb {
    async fn insert_user_white_list(&self, user_white_list: &UserWhiteList) -> Result<()> {
        let mut user_white_lists = self.user_white_lists.lock().await;
        if user_white_lists.contains_key(&user_white_list.email) {
            Err(create_database_error!("insert", "user_white_list"))
        } else {
            user_white_lists.insert(user_white_list.email.to_string(), user_white_list.clone());
            Ok(())
        }
    }

    async fn fetch_user_white_list(&self, email: &str) -> Result<UserWhiteList> {
        let user_white_lists = self.user_white_lists.lock().await;
        user_white_lists
            .get(email)
            .cloned()
            .ok_or_else(|| create_error!(NotFound))
    }

    async fn fetch_user_white_lists<'a>(&self) -> Result<Vec<UserWhiteList>> {
        let user_white_lists = self.user_white_lists.lock().await;
        Ok(user_white_lists.values().cloned().collect())
    }
}
