use revolt_result::Result;

use crate::ReferenceDb;
use crate::{AdminStrike, PartialAdminStrike};

use super::AbstractAdminStrikes;

#[async_trait]
impl AbstractAdminStrikes for ReferenceDb {
    async fn admin_strike_insert(&self, strike: AdminStrike) -> Result<()> {
        let mut admin_strikes = self.admin_strikes.lock().await;
        if admin_strikes.contains_key(&strike.id) {
            Err(create_database_error!("insert", "admin_strikes"))
        } else {
            admin_strikes.insert(strike.id.to_string(), strike.clone());
            Ok(())
        }
    }

    async fn admin_strike_update(
        &self,
        strike_id: &str,
        partial: PartialAdminStrike,
    ) -> Result<()> {
        let mut admin_strikes = self.admin_strikes.lock().await;
        if let Some(strike) = admin_strikes.get_mut(strike_id) {
            strike.apply_options(partial);
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }

    async fn admin_strike_get(&self, strike_id: &str) -> Result<AdminStrike> {
        let admin_strikes = self.admin_strikes.lock().await;
        if let Some(strike) = admin_strikes.get(strike_id) {
            Ok(strike.clone())
        } else {
            Err(create_error!(NotFound))
        }
    }

    async fn admin_strike_get_user(&self, user_id: &str) -> Result<Vec<AdminStrike>> {
        let admin_strikes = self.admin_strikes.lock().await;
        Ok(admin_strikes
            .iter()
            .filter(|(_, strike)| strike.target_id == user_id)
            .map(|(_, strike)| strike.clone())
            .collect())
    }
}
