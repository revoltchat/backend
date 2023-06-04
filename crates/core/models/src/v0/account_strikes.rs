auto_derived!(
    /// Account Strike
    pub struct AccountStrike {
        /// Strike Id
        #[cfg_attr(feature = "serde", serde(rename = "_id"))]
        pub id: String,
        /// Id of reported user
        pub user_id: String,

        /// Attached reason
        pub reason: String,
    }

    /// New strike information
    pub struct DataCreateStrike {
        /// Id of reported user
        pub user_id: String,

        /// Attached reason
        pub reason: String,
    }

    /// New strike information
    pub struct DataEditAccountStrike {
        /// New attached reason
        pub reason: String,
    }
);
