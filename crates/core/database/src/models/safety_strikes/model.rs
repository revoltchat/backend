use revolt_result::Result;

use crate::Database;

auto_derived_partial!(
    /// Account Strike
    pub struct AccountStrike {
        /// Strike Id
        #[serde(rename = "_id")]
        pub id: String,
        /// Id of reported user
        pub user_id: String,
        /// Id of moderator
        pub moderator_id: String,

        /// Attached reason
        pub reason: String,
    },
    "PartialAccountStrike"
);

#[allow(clippy::disallowed_methods)]
impl AccountStrike {
    pub async fn create(
        db: &Database,
        user_id: String,
        reason: String,
        moderator_id: String,
    ) -> Result<AccountStrike> {
        let strike = AccountStrike {
            id: ulid::Ulid::new().to_string(),
            user_id,
            moderator_id,
            reason,
        };

        db.insert_account_strike(&strike).await?;
        Ok(strike)
    }

    /// Update this strike
    pub async fn update(&mut self, db: &Database, partial: PartialAccountStrike) -> Result<()> {
        db.update_account_strike(&self.id, &partial).await?;
        self.apply_options(partial);
        Ok(())
    }

    /// Delete this strike
    pub async fn delete(&self, db: &Database) -> Result<()> {
        db.delete_account_strike(&self.id).await
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{AccountStrike, PartialAccountStrike};

    #[async_std::test]
    async fn crud() {
        database_test!(|db| async move {
            let user_id = "user";

            let strike = AccountStrike::create(
                &db,
                user_id.to_string(),
                "reason 1".to_string(),
                "moderator_id".to_string(),
            )
            .await
            .unwrap();

            let mut updated_strike = strike.clone();
            updated_strike
                .update(
                    &db,
                    PartialAccountStrike {
                        reason: Some("new reason".to_string()),
                        ..Default::default()
                    },
                )
                .await
                .unwrap();

            let strike2 = AccountStrike::create(
                &db,
                user_id.to_string(),
                "reason 2".to_string(),
                "moderator_id".to_string(),
            )
            .await
            .unwrap();

            let strikes = db.fetch_account_strikes_by_user(user_id).await.unwrap();

            let ids = strikes
                .iter()
                .cloned()
                .map(|strike| strike.id)
                .collect::<HashSet<String>>();

            assert!(ids.contains(&strike.id));
            assert!(ids.contains(&strike2.id));

            let fetched_strike = strikes
                .into_iter()
                .find(|entry| entry.id == strike.id)
                .unwrap();

            assert_eq!(fetched_strike, updated_strike);
            assert_ne!(fetched_strike, strike);

            strike.delete(&db).await.unwrap();
            assert_eq!(
                1,
                db.fetch_account_strikes_by_user(user_id)
                    .await
                    .unwrap()
                    .len()
            )
        });
    }
}
