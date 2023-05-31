auto_derived!(
    /// Account Strike
    pub struct AccountStrike {
        /// Strike Id
        #[serde(rename = "_id")]
        pub id: String,
        /// User Id of reported user
        pub user_id: String,

        /// Attached reason
        pub reason: String,
    }

    /// # Strike Data
    pub struct DataEditAccountStrike {
        /// New attached reason
        pub reason: String,
    }
);

#[cfg(feature = "from_database")]
impl From<revolt_database::AccountStrike> for AccountStrike {
    fn from(value: revolt_database::AccountStrike) -> Self {
        AccountStrike {
            id: value.id,
            user_id: value.user_id,
            reason: value.reason,
        }
    }
}
