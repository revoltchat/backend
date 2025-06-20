auto_derived_partial! {
    pub struct AdminObjectNote {
        /// The ID of the note, which is the same as the ID of the object it's attached to
        #[serde(rename = "_id")]
        pub id: String,
        /// When the note was edited, in iso8601
        pub edited_at: String,
        /// The last user to edit the note
        pub last_edited_by_id: String,
        /// The content of the note
        pub content: String,
    },
    "PartialAdminObjectNote"
}

impl AdminObjectNote {
    pub fn new(object_id: &str, user_id: &str, content: &str) -> AdminObjectNote {
        AdminObjectNote {
            id: object_id.to_string(),
            edited_at: iso8601_timestamp::Timestamp::now_utc()
                .format_short()
                .to_string(),
            last_edited_by_id: user_id.to_string(),
            content: content.to_string(),
        }
    }
    pub fn to_partial(&self) -> PartialAdminObjectNote {
        PartialAdminObjectNote {
            id: Some(self.id.clone()),
            edited_at: Some(self.edited_at.clone()),
            last_edited_by_id: Some(self.last_edited_by_id.clone()),
            content: Some(self.content.clone()),
        }
    }
}
