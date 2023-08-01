auto_derived!(
    /// Channel Unread
    pub struct ChannelUnread {
        /// Composite key pointing to a user's view of a channel
        #[serde(rename = "_id")]
        pub id: ChannelCompositeKey,

        /// Id of the last message read in this channel by a user
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub last_id: Option<String>,
        /// Array of message ids that mention the user
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "Vec::is_empty", default)
        )]
        pub mentions: Vec<String>,
    }

    /// Composite primary key consisting of channel and user id
    #[derive(Hash)]
    pub struct ChannelCompositeKey {
        /// Channel Id
        pub channel: String,
        /// User Id
        pub user: String,
    }
);
