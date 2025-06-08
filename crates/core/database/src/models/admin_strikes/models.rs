auto_derived_partial! {
    pub struct AdminStrike {
        /// The strike ID
        #[serde(rename = "_id")]
        pub id: String,
        /// The object receiving the strike (user/server)
        pub target_id: String,
        /// The moderator who gave the strike
        pub mod_id: String,
        /// The case the strike was made under
        pub case_id: Option<String>,
        /// Action associated with the strike (eg. suspension/ban)
        pub associated_action: Option<String>,
        /// Has the strike been removed
        pub overruled: bool,
        /// The user-facing reason for the strike
        pub reason: String,
        /// Internal context for the strike
        pub mod_context: Option<String>,
    },
    "PartialAdminStrike"
}
