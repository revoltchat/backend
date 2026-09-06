use super::{Channel, File, Server, User};

auto_derived!(
    /// Invite
    #[serde(tag = "type")]
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
            /// Human-readable label, so an admin can tell codes apart
            #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
            label: Option<String>,
            /// How many times this code may be used; absent means unlimited
            #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
            max_uses: Option<u32>,
            /// How many times this code has been used
            #[cfg_attr(
                feature = "serde",
                serde(skip_serializing_if = "crate::if_zero_u32", default)
            )]
            uses: u32,
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

    /// Options when creating an invite
    #[derive(Default)]
    pub struct DataCreateInvite {
        /// Human-readable label so an admin can tell codes apart, e.g.
        /// "website", "Facebook post September", "migration".
        /// Requires ManageServer.
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub label: Option<String>,
        /// How many times the code may be used. Requires ManageServer.
        /// Omitted means one use.
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub max_uses: Option<u32>,
        /// Explicitly unlimited. Requires ManageServer.
        ///
        /// A separate flag rather than "max_uses omitted", so forgetting the
        /// field and meaning unlimited cannot look the same.
        #[cfg_attr(feature = "serde", serde(default))]
        pub unlimited: bool,
    }

    /// Public invite response
    #[allow(clippy::large_enum_variant)]
    #[serde(tag = "type")]
    pub enum InviteResponse {
        /// Server channel invite
        Server {
            /// Invite code
            code: String,
            /// Id of the server
            server_id: String,
            /// Name of the server
            server_name: String,
            /// Attachment for server icon
            #[serde(skip_serializing_if = "Option::is_none")]
            server_icon: Option<File>,
            /// Attachment for server banner
            #[serde(skip_serializing_if = "Option::is_none")]
            server_banner: Option<File>,
            /// Enum of server flags
            #[serde(skip_serializing_if = "Option::is_none")]
            server_flags: Option<i32>,
            /// Id of server channel
            channel_id: String,
            /// Name of server channel
            channel_name: String,
            /// Description of server channel
            #[serde(skip_serializing_if = "Option::is_none")]
            channel_description: Option<String>,
            /// Name of user who created the invite
            user_name: String,
            /// Avatar of the user who created the invite
            #[serde(skip_serializing_if = "Option::is_none")]
            user_avatar: Option<File>,
            /// Number of members in this server
            member_count: i64,
        },
        /// Group channel invite
        Group {
            /// Invite code
            code: String,
            /// Id of group channel
            channel_id: String,
            /// Name of group channel
            channel_name: String,
            /// Description of group channel
            #[serde(skip_serializing_if = "Option::is_none")]
            channel_description: Option<String>,
            /// Name of user who created the invite
            user_name: String,
            /// Avatar of the user who created the invite
            #[serde(skip_serializing_if = "Option::is_none")]
            user_avatar: Option<File>,
        },
    }

    /// Invite join response
    #[serde(tag = "type")]
    #[allow(clippy::large_enum_variant)]
    pub enum InviteJoinResponse {
        Server {
            /// Channels in the server
            channels: Vec<Channel>,
            /// Server we are joining
            server: Server,
        },
        Group {
            /// Group channel we are joining
            channel: Channel,
            /// Members of this group
            users: Vec<User>,
        },
    }
);
