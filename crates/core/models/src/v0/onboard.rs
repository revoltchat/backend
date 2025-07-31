auto_derived!(
    /// # New User Data
    #[derive(validator::Validate)]
    pub struct DataOnboard {
        /// New username which will be used to identify the user on the platform
        #[validate(length(min = 2, max = 32), regex = "super::RE_USERNAME")]
        pub username: String,
    }

    /// # Onboarding Status
    pub struct DataHello {
        /// Whether onboarding is required
        pub onboarding: bool,
    }
);