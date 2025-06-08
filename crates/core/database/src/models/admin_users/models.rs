auto_derived_partial! {
    pub struct AdminUser {
        /// The ID of the user
        #[serde(rename = "_id")]
        pub id: String,
        /// The user's email
        pub email: String,
        /// Whether the user is active or not (ie. can they use the api)
        pub active: bool,
        /// The permissions of the user
        pub permissions: u64,
    },
    "PartialAdminUser"
}
