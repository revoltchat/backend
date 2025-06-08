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
        pub valid: bool
    }
}

impl AdminMachineToken {
    pub fn new() -> AdminMachineToken {
        AdminMachineToken { valid: true }
    }
}

impl Default for AdminMachineToken {
    fn default() -> Self {
        AdminMachineToken::new()
    }
}
