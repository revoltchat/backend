use revolt_models::v0::{Message, Server, User};

auto_derived!(
    /// Snapshot of some content
    pub struct Snapshot {
        /// Unique Id
        #[serde(rename = "_id")]
        pub id: String,
        /// Report parent Id
        pub report_id: String,
        /// Snapshot of content
        pub content: SnapshotContent,
    }

    /// Enum to map into different models
    /// that can be saved in a snapshot
    #[serde(tag = "_type")]
    pub enum SnapshotContent {
        Message {
            /// Context before the message
            #[serde(rename = "_prior_context", default)]
            prior_context: Vec<Message>,

            /// Context after the message
            #[serde(rename = "_leading_context", default)]
            leading_context: Vec<Message>,

            /// Message
            #[serde(flatten)]
            message: Message,
        },
        Server(Server),
        User(User),
    }
);
