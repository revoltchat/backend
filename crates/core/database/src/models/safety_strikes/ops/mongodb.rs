use futures::StreamExt;
use revolt_result::Result;

use crate::MongoDb;
use crate::{AccountStrike, PartialAccountStrike};

use super::AbstractAccountStrikes;

static COL: &str = "safety_strikes";

#[async_trait]
impl AbstractAccountStrikes for MongoDb {
    /// Insert new strike into the database
    async fn insert_account_strike(&self, strike: &AccountStrike) -> Result<()> {
        query!(self, insert_one, COL, &strike).map(|_| ())
    }

    /// Fetch strike by id
    async fn fetch_account_strike(&self, id: &str) -> Result<AccountStrike> {
        query!(self, find_one_by_id, COL, id)?.ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch strikes by user id
    async fn fetch_account_strikes_by_user(&self, user_id: &str) -> Result<Vec<AccountStrike>> {
        Ok(self
            .col::<AccountStrike>(COL)
            .find(
                doc! {
                    "user_id": user_id,
                },
                None,
            )
            .await
            .map_err(|_| create_database_error!("find", COL))?
            .filter_map(|s| async {
                if cfg!(debug_assertions) {
                    Some(s.unwrap())
                } else {
                    s.ok()
                }
            })
            .collect()
            .await)
    }

    /// Update strike with new information
    async fn update_account_strike(&self, id: &str, partial: &PartialAccountStrike) -> Result<()> {
        query!(self, update_one_by_id, COL, id, partial, vec![], None).map(|_| ())
    }

    /// Delete a strike from the database
    async fn delete_account_strike(&self, id: &str) -> Result<()> {
        query!(self, delete_one_by_id, COL, id).map(|_| ())
    }
}
