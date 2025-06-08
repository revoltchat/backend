use revolt_result::Result;

use crate::MongoDb;
use crate::{AdminStrike, PartialAdminStrike};

use super::AbstractAdminStrikes;

static COL: &str = "admin_strikes";

#[async_trait]
impl AbstractAdminStrikes for MongoDb {
    async fn admin_strike_insert(&self, strike: AdminStrike) -> Result<()> {
        query!(self, insert_one, COL, strike).map(|_| ())
    }

    async fn admin_strike_update(
        &self,
        strike_id: &str,
        partial: PartialAdminStrike,
    ) -> Result<()> {
        query!(
            self,
            update_one_by_id,
            COL,
            strike_id,
            partial,
            vec![],
            None
        )
        .map(|_| ())
    }

    async fn admin_strike_get(&self, strike_id: &str) -> Result<AdminStrike> {
        query!(self, find_one_by_id, COL, strike_id)?
            .ok_or_else(|| create_database_error!("find_one", COL))?
    }

    async fn admin_strike_get_user(&self, user_id: &str) -> Result<Vec<AdminStrike>> {
        query!(self, find, COL, doc! {"target_id": user_id})
    }
}
