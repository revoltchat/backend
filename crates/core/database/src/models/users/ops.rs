use revolt_result::Result;

use crate::User;

mod mongodb;
mod reference;

#[async_trait]
pub trait AbstractUsers: Sync + Send {
    /// Fetch a user from the database
    async fn fetch_user(&self, id: &str) -> Result<User>;
}
