use super::File;

use revolt_permissions::{Override, OverrideField};
use std::collections::{HashMap, HashSet};

#[cfg(feature = "rocket")]
use rocket::FromForm;

auto_derived!(
    /// Channel
    #[serde(tag = "channel_type")]
    pub enum Channel {
        /// Personal "Saved Notes" channel which allows users to save messages
        SavedMessages {
            /// Unique Id
            #[cfg_attr(feature = "serde", serde(rename = "_id"))]
            id: String,
            /// Id of the user this channel belongs to
            user: String,
        },
        /// Direct message channel between two users
        DirectMessage {
            /// Unique Id
            #[cfg_attr(feature = "serde", serde(rename = "_id"))]
            id: String,

            /// Whether this direct message channel is currently open on both sides
            active: bool,
            /// 2-tuple of user ids participating in direct message
            recipients: Vec<String>,
            /// Id of the last message sent in this channel
            #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
            last_message_id: Option<String>,
        },
        /// Group channel between 1 or more participants
        Group {
            /// Unique Id
            #[cfg_attr(feature = "serde", serde(rename = "_id"))]
            id: String,

            /// Display name of the channel
            name: String,
            /// User id of the owner of the group
            owner: String,
            /// Channel description
            #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
            description: Option<String>,
            /// Array of user ids participating in channel
            recipients: Vec<String>,

            /// Custom icon attachment
            #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
            icon: Option<File>,
            /// Id of the last message sent in this channel
            #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
            last_message_id: Option<String>,

            /// Permissions assigned to members of this group
            /// (does not apply to the owner of the group)
            #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
            permissions: Option<i64>,

            /// Whether this group is marked as not safe for work
            #[cfg_attr(
                feature = "serde",
                serde(skip_serializing_if = "crate::if_false", default)
            )]
            nsfw: bool,
        },
        /// Text channel belonging to a server
        TextChannel {
            /// Unique Id
            #[cfg_attr(feature = "serde", serde(rename = "_id"))]
            id: String,
            /// Id of the server this channel belongs to
            server: String,

            /// Display name of the channel
            name: String,
            /// Channel description
            #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
            description: Option<String>,

            /// Custom icon attachment
            #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
            icon: Option<File>,
            /// Id of the last message sent in this channel
            #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
            last_message_id: Option<String>,

            /// Default permissions assigned to users in this channel
            #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
            default_permissions: Option<OverrideField>,
            /// Permissions assigned based on role to this channel
            #[cfg_attr(
                feature = "serde",
                serde(
                    default = "HashMap::<String, OverrideField>::new",
                    skip_serializing_if = "HashMap::<String, OverrideField>::is_empty"
                )
            )]
            role_permissions: HashMap<String, OverrideField>,

            /// Whether this channel is marked as not safe for work
            #[cfg_attr(
                feature = "serde",
                serde(skip_serializing_if = "crate::if_false", default)
            )]
            nsfw: bool,
        },
        /// Voice channel belonging to a server
        VoiceChannel {
            /// Unique Id
            #[cfg_attr(feature = "serde", serde(rename = "_id"))]
            id: String,
            /// Id of the server this channel belongs to
            server: String,

            /// Display name of the channel
            name: String,
            #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
            /// Channel description
            description: Option<String>,
            /// Custom icon attachment
            #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
            icon: Option<File>,

            /// Default permissions assigned to users in this channel
            #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
            default_permissions: Option<OverrideField>,
            /// Permissions assigned based on role to this channel
            #[cfg_attr(
                feature = "serde",
                serde(
                    default = "HashMap::<String, OverrideField>::new",
                    skip_serializing_if = "HashMap::<String, OverrideField>::is_empty"
                )
            )]
            role_permissions: HashMap<String, OverrideField>,

            /// Whether this channel is marked as not safe for work
            #[cfg_attr(
                feature = "serde",
                serde(skip_serializing_if = "crate::if_false", default)
            )]
            nsfw: bool,
        },
    }

    /// Partial representation of a channel
    #[derive(Default)]
    pub struct PartialChannel {
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub name: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub owner: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub description: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub icon: Option<File>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub nsfw: Option<bool>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub active: Option<bool>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub permissions: Option<i64>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub role_permissions: Option<HashMap<String, OverrideField>>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub default_permissions: Option<OverrideField>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub last_message_id: Option<String>,
    }

    /// Optional fields on channel object
    pub enum FieldsChannel {
        Description,
        Icon,
        DefaultPermissions,
    }

    /// New webhook information
    #[cfg_attr(feature = "validator", derive(validator::Validate))]
    pub struct DataEditChannel {
        /// Channel name
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 32)))]
        pub name: Option<String>,

        /// Channel description
        #[cfg_attr(feature = "validator", validate(length(min = 0, max = 1024)))]
        pub description: Option<String>,

        /// Group owner
        pub owner: Option<String>,

        /// Icon
        ///
        /// Provide an Autumn attachment Id.
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 128)))]
        pub icon: Option<String>,

        /// Whether this channel is age-restricted
        pub nsfw: Option<bool>,

        /// Whether this channel is archived
        pub archived: Option<bool>,

        /// Fields to remove from channel
        #[cfg_attr(feature = "serde", serde(default))]
        pub remove: Vec<FieldsChannel>,
    }

    /// Create new group
    #[derive(Default)]
    #[cfg_attr(feature = "validator", derive(validator::Validate))]
    pub struct DataCreateGroup {
        /// Group name
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 32)))]
        pub name: String,
        /// Group description
        #[cfg_attr(feature = "validator", validate(length(min = 0, max = 1024)))]
        pub description: Option<String>,
        /// Group icon
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 128)))]
        pub icon: Option<String>,
        /// Array of user IDs to add to the group
        ///
        /// Must be friends with these users.
        #[cfg_attr(feature = "validator", validate(length(min = 0, max = 49)))]
        #[serde(default)]
        pub users: HashSet<String>,
        /// Whether this group is age-restricted
        #[serde(skip_serializing_if = "Option::is_none")]
        pub nsfw: Option<bool>,
    }

    /// Server Channel Type
    #[derive(Default)]
    pub enum LegacyServerChannelType {
        /// Text Channel
        #[default]
        Text,
        /// Voice Channel
        Voice,
    }

    /// Create new server channel
    #[derive(Default)]
    #[cfg_attr(feature = "validator", derive(validator::Validate))]
    pub struct DataCreateServerChannel {
        /// Channel type
        #[serde(rename = "type", default = "LegacyServerChannelType::default")]
        pub channel_type: LegacyServerChannelType,
        /// Channel name
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 32)))]
        pub name: String,
        /// Channel description
        #[cfg_attr(feature = "validator", validate(length(min = 0, max = 1024)))]
        pub description: Option<String>,
        /// Whether this channel is age restricted
        #[serde(skip_serializing_if = "Option::is_none")]
        pub nsfw: Option<bool>,
    }

    /// New default permissions
    #[serde(untagged)]
    pub enum DataDefaultChannelPermissions {
        Value {
            /// Permission values to set for members in a `Group`
            permissions: u64,
        },
        Field {
            /// Allow / deny values to set for members in this `TextChannel` or `VoiceChannel`
            permissions: Override,
        },
    }

    /// New role permissions
    pub struct DataSetRolePermissions {
        /// Allow / deny values to set for this role
        pub permissions: Override,
    }

    /// Options when deleting a channel
    #[cfg_attr(feature = "rocket", derive(FromForm))]
    pub struct OptionsChannelDelete {
        /// Whether to not send a leave message
        pub leave_silently: Option<bool>,
    }

    /// Voice server token response
    pub struct LegacyCreateVoiceUserResponse {
        /// Token for authenticating with the voice server
        token: String,
    }
);

impl Channel {
    /// Get a reference to this channel's id
    pub fn id(&self) -> &str {
        match self {
            Channel::DirectMessage { id, .. }
            | Channel::Group { id, .. }
            | Channel::SavedMessages { id, .. }
            | Channel::TextChannel { id, .. }
            | Channel::VoiceChannel { id, .. } => id,
        }
    }

    /// This returns a Result because the recipient name can't be determined here without a db call,
    /// which can't be done since this is models, which can't reference the database crate.
    ///
    /// If it returns None, you need to fetch the name from the db.
    pub fn name(&self) -> Option<&str> {
        match self {
            Channel::DirectMessage { .. } => None,
            Channel::SavedMessages { .. } => Some("Saved Messages"),
            Channel::TextChannel { name, .. }
            | Channel::Group { name, .. }
            | Channel::VoiceChannel { name, .. } => Some(name),
        }
    }
}
