mod mongodb;
mod reference;
use revolt_result::Result;

use crate::models::admin_strikes::models::{AdminStrike, PartialAdminStrike};

#[async_trait]
pub trait AbstractAdminStrikes: Sync + Send {
    async fn admin_strike_insert(&self, strike: AdminStrike) -> Result<()>;

    async fn admin_strike_update(&self, strike_id: &str, partial: PartialAdminStrike)
        -> Result<()>;

    async fn admin_strike_get(&self, strike_id: &str) -> Result<AdminStrike>;

    async fn admin_strike_get_user(&self, user_id: &str) -> Result<Vec<AdminStrike>>;
}
