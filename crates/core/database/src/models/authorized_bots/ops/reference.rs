use bson::Document;
use revolt_result::Result;

use crate::{ReferenceDb, AuthorizedBot, AuthorizedBotId};

use super::AbstractAuthorizedBots;

static COL: &str = "authorized_bots";

#[async_trait]
impl AbstractAuthorizedBots for ReferenceDb {
        /// Insert emoji into database.
    async fn insert_authorized_bot(&self, authorised_bot: &AuthorizedBot) -> Result<()> {
        todo!()
    }

    /// Fetch an emoji by its id
    async fn fetch_authorized_bot(&self, id: &AuthorizedBotId) -> Result<AuthorizedBot> {
        todo!()
    }

    async fn fetch_users_authorized_bots(&self, user_id: &str) -> Result<Vec<AuthorizedBot>> {
        todo!()
    }

    /// Deletes an authori
    async fn delete_authorized_bot(&self, id: &AuthorizedBotId) -> Result<()> {
        todo!()
    }

    async fn deauthorize_authorized_bot(&self, id: &AuthorizedBotId) -> Result<AuthorizedBot> {
        todo!()
    }

    async fn fetch_deauthorized_authorized_bots(&self) -> Result<Vec<AuthorizedBot>> {
        todo!()
    }
}