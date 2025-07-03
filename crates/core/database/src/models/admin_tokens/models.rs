use iso8601_timestamp::Timestamp;
use revolt_result::Result;

use crate::{AdminUser, Database};

auto_derived! {
    pub struct AdminToken {
        /// The token ID
        #[serde(rename = "_id")]
        pub id: String,
        /// The user this token is attached to
        pub user_id: String,
        /// The token itself
        pub token: String,
        /// The expiry timestamp for this token, in iso6801
        pub expiry: String
    }

    /// This struct is used to validate machine tokens when doing machine to machine communication.
    pub struct AdminMachineToken {
        /// Placeholder field.
        pub on_behalf_of: AdminUser
    }

    pub enum AdminAuthorization {
        AdminUser(AdminUser),
        AdminMachine(AdminMachineToken),
    }
}

impl AdminToken {
    pub fn new(user_id: &str, expiry: Timestamp) -> AdminToken {
        let id = ulid::Ulid::new().to_string();
        let token = nanoid::nanoid!(64);
        AdminToken {
            id,
            user_id: user_id.to_string(),
            token,
            expiry: expiry.format_short().to_string(),
        }
    }
}

impl AdminMachineToken {
    pub async fn new_from_email(email: &str, db: &Database) -> Result<AdminMachineToken> {
        println!("{}", email);
        if email == "example@example.com" {
            // This is basically just a workaround to make the first user.
            let user_count = db.admin_user_count().await?;
            println!("{}", user_count);
            if user_count == 0 {
                return Ok(AdminMachineToken {
                    on_behalf_of: AdminUser {
                        id: "00".to_string(),
                        platform_user_id: "".to_string(),
                        email: "example@example.com".to_string(),
                        active: true,
                        permissions: u64::MAX,
                    },
                });
            }
        }
        let user = db.admin_user_fetch_email(email).await?;
        Ok(AdminMachineToken { on_behalf_of: user })
    }

    pub async fn new_from_id(user_id: &str, db: &Database) -> Result<AdminMachineToken> {
        let user = db.admin_user_fetch(user_id).await?;
        Ok(AdminMachineToken { on_behalf_of: user })
    }
}
