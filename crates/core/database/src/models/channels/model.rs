use std::collections::HashMap;

use revolt_permissions::OverrideField;
use revolt_result::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Database, File, IntoDocumentPath};

/// Utility function to check if a boolean value is false
pub fn if_false(t: &bool) -> bool {
    !t
}

auto_derived!(
    pub enum Channel {
        /// Personal "Saved Notes" channel which allows users to save messages
        SavedMessages {
            /// Unique Id
            #[serde(rename = "_id")]
            id: String,
            /// Id of the user this channel belongs to
            user: String,
        },
        /// Direct message channel between two users
        DirectMessage {
            /// Unique Id
            #[serde(rename = "_id")]
            id: String,

            /// Whether this direct message channel is currently open on both sides
            active: bool,
            /// 2-tuple of user ids participating in direct message
            recipients: Vec<String>,
            /// Id of the last message sent in this channel
            #[serde(skip_serializing_if = "Option::is_none")]
            last_message_id: Option<String>,
        },
        /// Group channel between 1 or more participants
        Group {
            /// Unique Id
            #[serde(rename = "_id")]
            id: String,

            /// Display name of the channel
            name: String,
            /// User id of the owner of the group
            owner: String,
            /// Channel description
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            /// Array of user ids participating in channel
            recipients: Vec<String>,

            /// Custom icon attachment
            #[serde(skip_serializing_if = "Option::is_none")]
            icon: Option<File>,
            /// Id of the last message sent in this channel
            #[serde(skip_serializing_if = "Option::is_none")]
            last_message_id: Option<String>,

            /// Permissions assigned to members of this group
            /// (does not apply to the owner of the group)
            #[serde(skip_serializing_if = "Option::is_none")]
            permissions: Option<i64>,

            /// Whether this group is marked as not safe for work
            #[serde(skip_serializing_if = "if_false", default)]
            nsfw: bool,
        },
        /// Text channel belonging to a server
        TextChannel {
            /// Unique Id
            #[serde(rename = "_id")]
            id: String,
            /// Id of the server this channel belongs to
            server: String,

            /// Display name of the channel
            name: String,
            /// Channel description
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,

            /// Custom icon attachment
            #[serde(skip_serializing_if = "Option::is_none")]
            icon: Option<File>,
            /// Id of the last message sent in this channel
            #[serde(skip_serializing_if = "Option::is_none")]
            last_message_id: Option<String>,

            /// Default permissions assigned to users in this channel
            #[serde(skip_serializing_if = "Option::is_none")]
            default_permissions: Option<OverrideField>,
            /// Permissions assigned based on role to this channel
            #[serde(
                default = "HashMap::<String, OverrideField>::new",
                skip_serializing_if = "HashMap::<String, OverrideField>::is_empty"
            )]
            role_permissions: HashMap<String, OverrideField>,

            /// Whether this channel is marked as not safe for work
            #[serde(skip_serializing_if = "if_false", default)]
            nsfw: bool,
        },
        /// Voice channel belonging to a server
        VoiceChannel {
            /// Unique Id
            #[serde(rename = "_id")]
            id: String,
            /// Id of the server this channel belongs to
            server: String,

            /// Display name of the channel
            name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            /// Channel description
            description: Option<String>,
            /// Custom icon attachment
            #[serde(skip_serializing_if = "Option::is_none")]
            icon: Option<File>,

            /// Default permissions assigned to users in this channel
            #[serde(skip_serializing_if = "Option::is_none")]
            default_permissions: Option<OverrideField>,
            /// Permissions assigned based on role to this channel
            #[serde(
                default = "HashMap::<String, OverrideField>::new",
                skip_serializing_if = "HashMap::<String, OverrideField>::is_empty"
            )]
            role_permissions: HashMap<String, OverrideField>,

            /// Whether this channel is marked as not safe for work
            #[serde(skip_serializing_if = "if_false", default)]
            nsfw: bool,
        },
    }
);

auto_derived_partial!(
    struct NullName {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub owner: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub icon: Option<File>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub nsfw: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub active: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub permissions: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub role_permissions: Option<HashMap<String, OverrideField>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub default_permissions: Option<OverrideField>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub last_message_id: Option<String>,
    },
    "PartialChannel"
);

/// Optional fields on channel object
#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq, Eq, Clone)]
pub enum FieldsChannel {
    Description,
    Icon,
    DefaultPermissions,
}

impl Channel {
    /// Create a channel
    pub async fn create(&self, db: &Database) -> Result<()> {
        db.insert_channel(self).await?;

        Ok(())
    }

    /// Add user to a group
    pub async fn add_user_to_group(&mut self, db: &Database, user: &str, by: &str) -> Result<()> {
        if let Channel::Group { recipients, .. } = self {
            if recipients.contains(&String::from(user)) {
                return Err(create_error!(AlreadyInGroup));
            }

            recipients.push(String::from(user));
        }

        match &self {
            Channel::Group { id, .. } => {
                db.add_user_to_group(id, user).await?;

                Ok(())
            }
            _ => Err(create_error!(InvalidOperation)),
        }
    }

    /// Map out whether it is a direct DM
    pub fn is_direct_dm(&self) -> bool {
        matches!(self, Channel::DirectMessage { .. })
    }

    // return an override role
    pub fn find_role(&self, role_id: &str) -> Option<&OverrideField> {
        match self {
            Channel::TextChannel {
                role_permissions, ..
            }
            | Channel::VoiceChannel {
                role_permissions, ..
            } => role_permissions.get(role_id),
            _ => None,
        }
    }

    pub fn contains_user(&self, user_id: &str) -> bool {
        match self {
            Channel::Group { recipients, .. } => recipients.contains(&String::from(user_id)),
            _ => false,
        }
    }

    pub fn users(&self) -> Result<Vec<String>> {
        match self {
            Channel::Group { recipients, .. } => Ok(recipients.to_owned()),
            _ => Err(create_error!(NotFound)),
        }
    }

    /// Get a reference to this channel's id
    pub fn id(&self) -> String {
        match self {
            Channel::DirectMessage { id, .. }
            | Channel::Group { id, .. }
            | Channel::SavedMessages { id, .. }
            | Channel::TextChannel { id, .. }
            | Channel::VoiceChannel { id, .. } => id.clone(),
        }
    }

    /// Set role permission on a channel
    pub async fn set_role_permission(
        &mut self,
        role: &str,
        permissions: OverrideField,
    ) -> Result<()> {
        match self {
            Channel::TextChannel {
                role_permissions, ..
            }
            | Channel::VoiceChannel {
                role_permissions, ..
            } => {
                if let Some(_) = role_permissions.get(role) {
                    role_permissions.remove(role);
                    role_permissions.insert(String::from(role), permissions);

                    let mut partial = PartialChannel::empty();
                    partial.role_permissions = Some(role_permissions.to_owned());

                    self.apply_options(partial);

                    Ok(())
                } else {
                    Err(create_error!(NotFound))
                }
            }
            _ => Err(create_error!(InvalidOperation)),
        }
    }

    /// Update channel data
    pub async fn update<'a>(
        &mut self,
        db: &Database,
        partial: PartialChannel,
        remove: Vec<FieldsChannel>,
    ) -> Result<()> {
        for field in &remove {
            self.remove_field(field);
        }

        self.apply_options(partial.clone());

        db.update_channel(&self.id(), &partial, remove.clone())
            .await?;

        Ok(())
    }

    /// Remove a field from Channel object
    pub fn remove_field(&mut self, field: &FieldsChannel) {
        match field {
            FieldsChannel::Description => match self {
                Self::Group { description, .. }
                | Self::TextChannel { description, .. }
                | Self::VoiceChannel { description, .. } => {
                    description.take();
                }
                _ => {}
            },
            FieldsChannel::Icon => match self {
                Self::Group { icon, .. }
                | Self::TextChannel { icon, .. }
                | Self::VoiceChannel { icon, .. } => {
                    icon.take();
                }
                _ => {}
            },
            FieldsChannel::DefaultPermissions => match self {
                Self::TextChannel {
                    default_permissions,
                    ..
                }
                | Self::VoiceChannel {
                    default_permissions,
                    ..
                } => {
                    default_permissions.take();
                }
                _ => {}
            },
        }
    }

    /// Apply partial channel to channel
    pub fn apply_options(&mut self, partial: PartialChannel) {
        // ! FIXME: maybe flatten channel object?
        match self {
            Self::DirectMessage { active, .. } => {
                if let Some(v) = partial.active {
                    *active = v;
                }
            }
            Self::Group {
                name,
                owner,
                description,
                icon,
                nsfw,
                permissions,
                ..
            } => {
                if let Some(v) = partial.name {
                    *name = v;
                }

                if let Some(v) = partial.owner {
                    *owner = v;
                }

                if let Some(v) = partial.description {
                    description.replace(v);
                }

                if let Some(v) = partial.icon {
                    icon.replace(v);
                }

                if let Some(v) = partial.nsfw {
                    *nsfw = v;
                }

                if let Some(v) = partial.permissions {
                    permissions.replace(v);
                }
            }
            Self::TextChannel {
                name,
                description,
                icon,
                nsfw,
                default_permissions,
                role_permissions,
                ..
            }
            | Self::VoiceChannel {
                name,
                description,
                icon,
                nsfw,
                default_permissions,
                role_permissions,
                ..
            } => {
                if let Some(v) = partial.name {
                    *name = v;
                }

                if let Some(v) = partial.description {
                    description.replace(v);
                }

                if let Some(v) = partial.icon {
                    icon.replace(v);
                }

                if let Some(v) = partial.nsfw {
                    *nsfw = v;
                }

                if let Some(v) = partial.role_permissions {
                    *role_permissions = v;
                }

                if let Some(v) = partial.default_permissions {
                    default_permissions.replace(v);
                }
            }
            _ => {}
        }
    }

    /// Acknowledge a message
    pub async fn ack(&self, user: &str, message: &str) -> Result<()> {
        //todo
        Ok(())
    }

    /// Remove user from a group
    pub async fn remove_user_from_group(
        &self,
        db: &Database,
        user: &str,
        by: Option<&str>,
        silent: bool,
    ) -> Result<()> {
        match &self {
            Channel::Group {
                id,
                owner,
                recipients,
                ..
            } => {
                if user == owner {
                    if let Some(new_owner) = recipients.iter().find(|x| *x != user) {
                        db.update_channel(
                            id,
                            &PartialChannel {
                                owner: Some(new_owner.into()),
                                ..Default::default()
                            },
                            vec![],
                        )
                        .await?;
                    } else {
                        db.delete_channel(self).await?;
                        return Ok(());
                    }
                }

                //todo send system message
                // afaik system messages are no longer supported by the Channel::Group type

                Ok(())
            }

            _ => Err(create_error!(InvalidOperation)),
        }
    }

    /// Delete a channel
    pub async fn delete(&self, db: &Database) -> Result<()> {
        db.delete_channel(&self).await
    }
}

impl IntoDocumentPath for FieldsChannel {
    fn as_path(&self) -> Option<&'static str> {
        Some(match self {
            FieldsChannel::Description => "description",
            FieldsChannel::Icon => "icon",
            FieldsChannel::DefaultPermissions => "default_permissions",
        })
    }
}
