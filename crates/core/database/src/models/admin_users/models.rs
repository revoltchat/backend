use revolt_result::Result;

use crate::Database;

auto_derived_partial! {
    pub struct AdminUser {
        /// The ID of the user
        #[serde(rename = "_id")]
        pub id: String,
        /// The user's revolt ID.
        pub platform_user_id: String,
        /// The user's email
        pub email: String,
        /// Whether the user is active or not (ie. can they use the api)
        pub active: bool,
        /// The permissions of the user
        pub permissions: u64,
    },
    "PartialAdminUser"
}

impl AdminUser {
    pub fn new(email: &str, platform_user_id: &str, permissions: u64) -> AdminUser {
        let id = ulid::Ulid::new().to_string();
        AdminUser {
            id,
            platform_user_id: platform_user_id.to_string(),
            email: email.to_string(),
            active: true,
            permissions,
        }
    }

    pub async fn find_by_id(id: &str, db: &Database) -> Result<AdminUser> {
        return db.admin_user_fetch(id).await;
    }

    pub async fn find_by_email(email: &str, db: &Database) -> Result<AdminUser> {
        return db.admin_user_fetch_email(email).await;
    }
}
