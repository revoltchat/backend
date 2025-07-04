use revolt_result::Result;

use crate::{AuthorizedBot, AuthorizedBotId};

mod mongodb;
mod reference;

#[async_trait]
pub trait AbstractAuthorizedBots: Sync + Send {
    /// Insert emoji into database.
    async fn insert_authorized_bot(&self, authorized_bot: &AuthorizedBot) -> Result<()>;

    /// Fetch an authorized bot by its id
    async fn fetch_authorized_bot(&self, id: &AuthorizedBotId) -> Result<AuthorizedBot>;

    /// Fetch a users authorized bot by its id
    async fn fetch_users_authorized_bots(&self, user_id: &str) -> Result<Vec<AuthorizedBot>>;

    /// Deletes an authorized bot
    async fn delete_authorized_bot(&self, id: &AuthorizedBotId) -> Result<()>;

    /// Deauthorizes an authorized bot
    async fn deauthorize_authorized_bot(&self, id: &AuthorizedBotId) -> Result<AuthorizedBot>;

    // Fetches all authorized bots which have been deauthorized
    async fn fetch_deauthorized_authorized_bots(&self) -> Result<Vec<AuthorizedBot>>;
}
