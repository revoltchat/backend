use std::collections::HashMap;

use revolt_models::v0::{self, MessageAuthor};
use revolt_permissions::OverrideField;
use revolt_result::Result;
use serde::{Deserialize, Serialize};

use crate::{events::client::EventV1, Database, File, IntoDocumentPath, SystemMessage, User};

auto_derived!(
    #[serde(tag = "channel_type")]
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
            #[serde(skip_serializing_if = "crate::if_false", default)]
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
            #[serde(skip_serializing_if = "crate::if_false", default)]
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
            #[serde(skip_serializing_if = "crate::if_false", default)]
            nsfw: bool,
        },
    }
);

auto_derived!(
    #[derive(Default)]
    pub struct PartialChannel {
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
    }

    /// Optional fields on channel object
    pub enum FieldsChannel {
        Description,
        Icon,
        DefaultPermissions,
    }
);

#[allow(clippy::disallowed_methods)]
impl Channel {
    /* /// Create a channel
    pub async fn create(&self, db: &Database) -> Result<()> {
        db.insert_channel(self).await?;

        let event = EventV1::ChannelCreate(self.clone().into());
        match self {
            Self::SavedMessages { user, .. } => event.private(user.clone()).await,
            Self::DirectMessage { recipients, .. } | Self::Group { recipients, .. } => {
                for recipient in recipients {
                    event.clone().private(recipient.clone()).await;
                }
            }
            Self::TextChannel { server, .. } | Self::VoiceChannel { server, .. } => {
                event.p(server.clone()).await;
            }
        }

        Ok(())
    }*/

    /// Create a group
    pub async fn create_group(
        db: &Database,
        data: v0::DataCreateGroup,
        owner_id: String,
    ) -> Result<Channel> {
        let recipients = data.users.into_iter().collect::<Vec<String>>();
        let channel = Channel::Group {
            id: ulid::Ulid::new().to_string(),

            name: data.name,
            owner: owner_id,
            description: data.description,
            recipients: recipients.clone(),

            icon: None,
            last_message_id: None,

            permissions: None,

            nsfw: data.nsfw.unwrap_or(false),
        };

        db.insert_channel(&channel).await?;

        let event = EventV1::ChannelCreate(channel.clone().into());
        for recipient in recipients {
            event.clone().private(recipient).await;
        }

        Ok(channel)
    }

    /// Add user to a group
    pub async fn add_user_to_group(
        &mut self,
        db: &Database,
        user: &User,
        by_id: &str,
    ) -> Result<()> {
        if let Channel::Group { recipients, .. } = self {
            if recipients.contains(&String::from(&user.id)) {
                return Err(create_error!(AlreadyInGroup));
            }

            recipients.push(String::from(&user.id));
        }

        match &self {
            Channel::Group { id, .. } => {
                db.add_user_to_group(id, &user.id).await?;

                EventV1::ChannelGroupJoin {
                    id: id.to_string(),
                    user: user.id.to_string(),
                }
                .p(id.to_string())
                .await;

                EventV1::ChannelCreate(self.clone().into())
                    .private(user.id.to_string())
                    .await;

                SystemMessage::UserAdded {
                    id: user.id.to_string(),
                    by: by_id.to_string(),
                }
                .into_message(id.to_string())
                .send(
                    db,
                    MessageAuthor::System {
                        username: &user.username,
                        avatar: user.avatar.as_ref().map(|file| file.id.as_ref()),
                    },
                    self,
                    false,
                )
                .await
                .ok();

                Ok(())
            }
            _ => Err(create_error!(InvalidOperation)),
        }
    }

    /// Map out whether it is a direct DM
    pub fn is_direct_dm(&self) -> bool {
        matches!(self, Channel::DirectMessage { .. })
    }

    /// Check whether has a user as a recipient
    pub fn contains_user(&self, user_id: &str) -> bool {
        match self {
            Channel::Group { recipients, .. } => recipients.contains(&String::from(user_id)),
            _ => false,
        }
    }

    /// Get list of recipients
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
        db: &Database,
        role_id: &str,
        permissions: OverrideField,
    ) -> Result<()> {
        match self {
            Channel::TextChannel {
                id,
                server,
                role_permissions,
                ..
            }
            | Channel::VoiceChannel {
                id,
                server,
                role_permissions,
                ..
            } => {
                db.set_channel_role_permission(id, role_id, permissions)
                    .await?;

                role_permissions.insert(role_id.to_string(), permissions);

                EventV1::ChannelUpdate {
                    id: id.clone(),
                    data: PartialChannel {
                        role_permissions: Some(role_permissions.clone()),
                        ..Default::default()
                    }
                    .into(),
                    clear: vec![],
                }
                .p(server.clone())
                .await;

                Ok(())
            }
            _ => Err(create_error!(InvalidOperation)),
        }
    }

    /// Update channel data
    pub async fn update(
        &mut self,
        db: &Database,
        partial: PartialChannel,
        remove: Vec<FieldsChannel>,
    ) -> Result<()> {
        for field in &remove {
            self.remove_field(field);
        }

        self.apply_options(partial.clone());

        let id = self.id().to_string();
        db.update_channel(&id, &partial, remove.clone()).await?;

        EventV1::ChannelUpdate {
            id: id.clone(),
            data: partial.into(),
            clear: remove.into_iter().map(|v| v.into()).collect(),
        }
        .p(match self {
            Self::TextChannel { server, .. } | Self::VoiceChannel { server, .. } => server.clone(),
            _ => id,
        })
        .await;

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

    /// Remove multiple fields from Channel object
    pub fn remove_fields(&mut self, partial: Vec<FieldsChannel>) {
        for field in partial {
            self.remove_field(&field)
        }
    }

    /// Apply partial channel to channel
    pub fn apply_options(&mut self, partial: PartialChannel) {
        match self {
            Self::SavedMessages { .. } => {}
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
        }
    }

    /// Remove user from a group
    pub async fn remove_user_from_group(
        &self,
        db: &Database,
        user: &User,
        by_id: Option<&str>,
        silent: bool,
    ) -> Result<()> {
        match &self {
            Channel::Group {
                id,
                name,
                owner,
                recipients,
                ..
            } => {
                if &user.id == owner {
                    if let Some(new_owner) = recipients.iter().find(|x| *x != &user.id) {
                        db.update_channel(
                            id,
                            &PartialChannel {
                                owner: Some(new_owner.into()),
                                ..Default::default()
                            },
                            vec![],
                        )
                        .await?;

                        SystemMessage::ChannelOwnershipChanged {
                            from: owner.to_string(),
                            to: new_owner.to_string(),
                        }
                        .into_message(id.to_string())
                        .send(
                            db,
                            MessageAuthor::System {
                                username: name,
                                avatar: None,
                            },
                            self,
                            false,
                        )
                        .await
                        .ok();
                    } else {
                        db.delete_channel(self).await?;
                        return Ok(());
                    }
                }

                EventV1::ChannelGroupLeave {
                    id: id.to_string(),
                    user: user.id.to_string(),
                }
                .p(id.to_string())
                .await;

                if !silent {
                    if let Some(by) = by_id {
                        SystemMessage::UserRemove {
                            id: user.id.to_string(),
                            by: by.to_string(),
                        }
                    } else {
                        SystemMessage::UserLeft {
                            id: user.id.to_string(),
                        }
                    }
                    .into_message(id.to_string())
                    .send(
                        db,
                        MessageAuthor::System {
                            username: &user.username,
                            avatar: user.avatar.as_ref().map(|file| file.id.as_ref()),
                        },
                        self,
                        false,
                    )
                    .await
                    .ok();
                }

                Ok(())
            }

            _ => Err(create_error!(InvalidOperation)),
        }
    }

    /// Delete a channel
    pub async fn delete(&self, db: &Database) -> Result<()> {
        let id = self.id().to_string();
        EventV1::ChannelDelete { id: id.clone() }.p(id).await;
        db.delete_channel(self).await
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
