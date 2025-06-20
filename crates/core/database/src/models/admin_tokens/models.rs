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
}

impl AdminMachineToken {
    pub async fn new_from_email(email: &str, db: &Database) -> Result<AdminMachineToken> {
        let user = db.admin_user_fetch_email(email).await?;
        Ok(AdminMachineToken { on_behalf_of: user })
    }

    pub async fn new_from_id(user_id: &str, db: &Database) -> Result<AdminMachineToken> {
        let user = db.admin_user_fetch(user_id).await?;
        Ok(AdminMachineToken { on_behalf_of: user })
    }
}
