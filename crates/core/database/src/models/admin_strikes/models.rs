use crate::util::basic::transform_optional_string;

auto_derived_partial! {
    pub struct AdminStrike {
        /// The strike ID
        #[serde(rename = "_id")]
        pub id: String,
        /// The object receiving the strike (user/server)
        pub target_id: String,
        /// The moderator who gave the strike
        pub mod_id: String,
        /// The case the strike was made under
        pub case_id: Option<String>,
        /// Action associated with the strike (eg. suspension/ban)
        pub associated_action: Option<String>,
        /// Has the strike been removed
        pub overruled: bool,
        /// The user-facing reason for the strike
        pub reason: String,
        /// Internal context for the strike
        pub mod_context: Option<String>,
    },
    "PartialAdminStrike"
}

impl AdminStrike {
    pub fn new(
        target_id: &str,
        mod_id: &str,
        case_id: Option<&str>,
        associated_action: Option<&str>,
        reason: &str,
        mod_context: Option<&str>,
    ) -> AdminStrike {
        let id = ulid::Ulid::new().to_string();
        AdminStrike {
            id,
            target_id: target_id.to_string(),
            mod_id: mod_id.to_string(),
            case_id: transform_optional_string(case_id),
            associated_action: transform_optional_string(associated_action),
            overruled: false,
            reason: reason.to_string(),
            mod_context: transform_optional_string(mod_context),
        }
    }
}
