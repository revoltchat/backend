use revolt_result::Result;

use crate::MongoDb;
use crate::User;

use super::AbstractUsers;

static COL: &str = "bots";

#[async_trait]
impl AbstractUsers for MongoDb {
    /// Fetch a user from the database
    async fn fetch_user(&self, id: &str) -> Result<User> {
        query!(self, find_one_by_id, COL, id)?.ok_or_else(|| create_error!(NotFound))
    }
}
