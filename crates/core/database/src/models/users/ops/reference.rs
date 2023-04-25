use revolt_result::Result;

use crate::ReferenceDb;
use crate::User;

use super::AbstractUsers;

#[async_trait]
impl AbstractUsers for ReferenceDb {
    /// Fetch a user from the database
    async fn fetch_user(&self, id: &str) -> Result<User> {
        let users = self.users.lock().await;
        users
            .get(id)
            .cloned()
            .ok_or_else(|| create_error!(NotFound))
    }
}
