use crate::MemberCompositeKey;

auto_derived!(
    /// Server Ban
    pub struct ServerBan {
        /// Unique member id
        #[serde(rename = "_id")]
        pub id: MemberCompositeKey,
        /// Reason for ban creation
        pub reason: Option<String>,
    }
);
