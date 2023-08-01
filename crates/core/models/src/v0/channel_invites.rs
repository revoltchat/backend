auto_derived!(
    /// Invite
    pub enum Invite {
        /// Invite to a specific server channel
        Server {
            /// Invite code
            #[cfg_attr(feature = "serde", serde(rename = "_id"))]
            code: String,
            /// Id of the server this invite points to
            server: String,
            /// Id of user who created this invite
            creator: String,
            /// Id of the server channel this invite points to
            channel: String,
        },
        /// Invite to a group channel
        Group {
            /// Invite code
            #[cfg_attr(feature = "serde", serde(rename = "_id"))]
            code: String,
            /// Id of user who created this invite
            creator: String,
            /// Id of the group channel this invite points to
            channel: String,
        },
    }
);
