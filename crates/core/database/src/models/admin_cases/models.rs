auto_derived_partial! {
    pub struct AdminCase {
        /// The case ID
        #[serde(rename = "_id")]
        pub id: String,
        /// The case Short ID
        pub short_id: String,

        /// The owner of the case
        pub owner_id: String,
        /// The title of the case
        pub title: String,
        /// The status of the case (open/closed)
        pub status: String,
        /// When the case was closed, in iso8601
        pub closed_at: Option<String>,
        /// The tags for the case
        pub tags: Vec<String>,
    },
    "PartialAdminCase"
}
