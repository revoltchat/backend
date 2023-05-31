use revolt_result::Result;

use crate::{AccountStrike, PartialAccountStrike};

mod mongodb;
mod reference;

#[async_trait]
pub trait AbstractAccountStrikes: Sync + Send {
    /// Insert new strike into the database
    async fn insert_account_strike(&self, strike: &AccountStrike) -> Result<()>;

    /// Fetch strikes by user id
    async fn fetch_account_strikes_by_user(&self, user_id: &str) -> Result<Vec<AccountStrike>>;

    /// Update strike with new information
    async fn update_account_strike(&self, id: &str, partial: &PartialAccountStrike) -> Result<()>;

    /// Delete a strike from the database
    async fn delete_account_strike(&self, id: &str) -> Result<()>;
}
