use super::MemberCompositeKey;

auto_derived!(
    /// Server Ban
    pub struct ServerBan {
        /// Unique member id
        #[cfg_attr(feature = "serde", serde(rename = "_id"))]
        pub id: MemberCompositeKey,
        /// Reason for ban creation
        pub reason: Option<String>,
    }
);
