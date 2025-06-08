auto_derived_partial! {
    pub struct AdminCaseComment {
        /// The comment ID
        #[serde(rename = "_id")]
        pub id: String,
        /// The ID of the case this comment is attached to
        pub case_id: String,
        /// The author ID
        pub user_id: String,
        /// When the comment was edited, if applicable, in iso8601
        pub edited_at: Option<String>,
        /// The content
        pub content: String
    },
    "PartialAdminCaseComment"
}
