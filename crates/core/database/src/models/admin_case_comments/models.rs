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

impl AdminCaseComment {
    pub fn new(case_id: &str, user_id: &str, content: &str) -> AdminCaseComment {
        let id = ulid::Ulid::new().to_string();
        AdminCaseComment {
            id,
            case_id: case_id.to_string(),
            user_id: user_id.to_string(),
            edited_at: None,
            content: content.to_string(),
        }
    }

    /// Edit the comment, updating the edited_at time as well
    pub fn edit(&mut self, content: &str) {
        self.content = content.to_string();
        self.edited_at = Some(
            iso8601_timestamp::Timestamp::now_utc()
                .format_short()
                .to_string(),
        );
    }
}

impl PartialAdminCaseComment {
    pub fn new() -> PartialAdminCaseComment {
        PartialAdminCaseComment::default()
    }
    /// Edit the comment, updating the edited_at time as well
    pub fn edit(&mut self, content: &str) {
        self.content = Some(content.to_string());
        self.edited_at = Some(
            iso8601_timestamp::Timestamp::now_utc()
                .format_short()
                .to_string(),
        );
    }
}
