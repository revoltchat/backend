use std::collections;

use revolt_result::Result;

use crate::Database;

auto_derived_partial!(
    pub struct UserWhiteList {
        /// Name user white listed
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,

        /// Email
        pub email: String,

        /// Whether this server should be publicly discoverable
        #[serde(skip_serializing_if = "Option::is_none")]
        pub phone_number: Option<String>,
    },
    "PartialUserWhiteList"
);

impl UserWhiteList {
    pub async fn create(&self, db: &Database) -> Result<()> {
        db.insert_user_white_list(self).await
    }
}
