use revolt_result::Result;

use crate::ReferenceDb;
use crate::{AccountStrike, PartialAccountStrike};

use super::AbstractAccountStrikes;

#[async_trait]
impl AbstractAccountStrikes for ReferenceDb {
    /// Insert new strike into the database
    async fn insert_account_strike(&self, strike: &AccountStrike) -> Result<()> {
        let mut strikes = self.account_strikes.lock().await;
        if strikes.contains_key(&strike.id) {
            Err(create_database_error!("insert", "strike"))
        } else {
            strikes.insert(strike.id.to_string(), strike.clone());
            Ok(())
        }
    }

    /// Fetch strike by id
    async fn fetch_account_strike(&self, id: &str) -> Result<AccountStrike> {
        let strikes = self.account_strikes.lock().await;
        strikes
            .get(id)
            .cloned()
            .ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch strikes by user id
    async fn fetch_account_strikes_by_user(&self, user_id: &str) -> Result<Vec<AccountStrike>> {
        let strikes = self.account_strikes.lock().await;
        Ok(strikes
            .values()
            .filter(|strike| strike.user_id == user_id)
            .cloned()
            .collect())
    }

    /// Update strike with new information
    async fn update_account_strike(&self, id: &str, partial: &PartialAccountStrike) -> Result<()> {
        let mut strikes = self.account_strikes.lock().await;
        if let Some(strike) = strikes.get_mut(id) {
            strike.apply_options(partial.clone());
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }

    /// Delete a strike from the database
    async fn delete_account_strike(&self, id: &str) -> Result<()> {
        let mut strikes = self.account_strikes.lock().await;
        if strikes.remove(id).is_some() {
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }
}
