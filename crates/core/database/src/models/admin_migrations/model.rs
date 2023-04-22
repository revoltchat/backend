auto_derived!(
    /// Document representing migration information
    pub struct MigrationInfo {
        /// Unique Id
        #[serde(rename = "_id")]
        pub id: i32,
        /// Current database revision
        pub revision: i32,
    }
);
