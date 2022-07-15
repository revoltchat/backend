use std::collections::HashSet;

use serde_json::json;
use ulid::Ulid;
use validator::Validate;

use crate::{
    events::client::EventV1,
    models::{
        message::{
            AppendMessage, BulkMessageResponse, Interactions, PartialMessage, SendableEmbed,
            SystemMessage,
        },
        Channel, Emoji, Message, User,
    },
    presence::presence_filter_online,
    tasks::ack::AckEvent,
    types::{
        january::{Embed, Text},
        push::PushNotification,
    },
    Database, Error, Result,
};

impl Message {
    /// Create a message
    pub async fn create_no_web_push(
        &mut self,
        db: &Database,
        channel: &str,
        is_direct_dm: bool,
    ) -> Result<()> {
        db.insert_message(self).await?;

        // Fan out events
        EventV1::Message(self.clone()).p(channel.to_string()).await;

        // Update last_message_id
        crate::tasks::last_message_id::queue(
            channel.to_string(),
            self.id.to_string(),
            is_direct_dm,
        )
        .await;

        // Add mentions for affected users
        if let Some(mentions) = &self.mentions {
            for user in mentions {
                crate::tasks::ack::queue(
                    channel.to_string(),
                    user.to_string(),
                    AckEvent::AddMention {
                        ids: vec![self.id.to_string()],
                    },
                )
                .await;
            }
        }

        Ok(())
    }

    /// Create a message and Web Push events
    pub async fn create(
        &mut self,
        db: &Database,
        channel: &Channel,
        sender: Option<&User>,
    ) -> Result<()> {
        self.create_no_web_push(db, channel.id(), channel.is_direct_dm())
            .await?;

        // Push out Web Push notifications
        crate::tasks::web_push::queue(
            {
                let mut target_ids = vec![];
                match &channel {
                    Channel::DirectMessage { recipients, .. }
                    | Channel::Group { recipients, .. } => {
                        target_ids = (&recipients.iter().cloned().collect::<HashSet<String>>()
                            - &presence_filter_online(recipients).await)
                            .into_iter()
                            .collect::<Vec<String>>();
                    }
                    Channel::TextChannel { .. } => {
                        if let Some(mentions) = &self.mentions {
                            target_ids.append(&mut mentions.clone());
                        }
                    }
                    _ => {}
                };
                target_ids
            },
            json!(PushNotification::new(self.clone(), sender, channel.id())).to_string(),
        )
        .await;

        Ok(())
    }

    /// Update message data
    pub async fn update(&mut self, db: &Database, partial: PartialMessage) -> Result<()> {
        self.apply_options(partial.clone());
        db.update_message(&self.id, &partial).await?;

        EventV1::MessageUpdate {
            id: self.id.clone(),
            channel: self.channel.clone(),
            data: partial,
        }
        .p(self.channel.clone())
        .await;

        Ok(())
    }

    /// Append message data
    pub async fn append(
        db: &Database,
        id: String,
        channel: String,
        append: AppendMessage,
    ) -> Result<()> {
        db.append_message(&id, &append).await?;

        EventV1::MessageAppend {
            id,
            channel: channel.to_string(),
            append,
        }
        .p(channel)
        .await;

        Ok(())
    }

    /// Delete a message
    pub async fn delete(self, db: &Database) -> Result<()> {
        let file_ids: Vec<String> = self
            .attachments
            .map(|files| files.iter().map(|file| file.id.to_string()).collect())
            .unwrap_or_default();

        if !file_ids.is_empty() {
            db.mark_attachments_as_deleted(&file_ids).await?;
        }

        db.delete_message(&self.id).await?;

        EventV1::MessageDelete {
            id: self.id,
            channel: self.channel.clone(),
        }
        .p(self.channel)
        .await;
        Ok(())
    }

    /// Bulk delete messages
    pub async fn bulk_delete(db: &Database, channel: &str, ids: Vec<String>) -> Result<()> {
        db.delete_messages(channel, ids.clone()).await?;
        EventV1::BulkMessageDelete {
            channel: channel.to_string(),
            ids,
        }
        .p(channel.to_string())
        .await;
        Ok(())
    }

    /// Validate the sum of content of a message is under threshold
    pub fn validate_sum(
        content: &Option<String>,
        embeds: &Option<Vec<SendableEmbed>>,
    ) -> Result<()> {
        let mut running_total = 0;
        if let Some(content) = content {
            running_total += content.len();
        }

        if let Some(embeds) = embeds {
            for embed in embeds {
                if let Some(desc) = &embed.description {
                    running_total += desc.len();
                }
            }
        }

        if running_total <= 2000 {
            Ok(())
        } else {
            Err(Error::PayloadTooLarge)
        }
    }

    /// Add a reaction to a message
    pub async fn add_reaction(&self, db: &Database, user: &User, emoji: &str) -> Result<()> {
        // Check if the emoji is whitelisted
        if !self.interactions.can_use(emoji) {
            return Err(Error::InvalidOperation);
        }

        // Check if the emoji is usable by us
        if !Emoji::can_use(db, emoji).await? {
            return Err(Error::InvalidOperation);
        }

        // Send reaction event
        EventV1::MessageReact {
            id: self.id.to_string(),
            channel_id: self.channel.to_string(),
            user_id: user.id.to_string(),
            emoji_id: emoji.to_string(),
        }
        .p(self.channel.to_string())
        .await;

        // Add emoji
        db.add_reaction(&self.id, emoji, &user.id).await
    }

    /// Remove a reaction from a message
    pub async fn remove_reaction(&self, db: &Database, user: &str, emoji: &str) -> Result<()> {
        // Check if it actually exists
        let empty = if let Some(users) = self.reactions.get(emoji) {
            if !users.contains(user) {
                return Err(Error::NotFound);
            }

            users.len() == 1
        } else {
            return Err(Error::NotFound);
        };

        // Send reaction event
        EventV1::MessageUnreact {
            id: self.id.to_string(),
            channel_id: self.channel.to_string(),
            user_id: user.to_string(),
            emoji_id: emoji.to_string(),
        }
        .p(self.channel.to_string())
        .await;

        if empty {
            // If empty, remove the reaction entirely
            db.clear_reaction(&self.id, emoji).await
        } else {
            // Otherwise only remove that one reaction
            db.remove_reaction(&self.id, emoji, user).await
        }
    }

    /// Remove a reaction from a message
    pub async fn clear_reaction(&self, db: &Database, emoji: &str) -> Result<()> {
        // Send reaction event
        EventV1::MessageRemoveReaction {
            id: self.id.to_string(),
            channel_id: self.channel.to_string(),
            emoji_id: emoji.to_string(),
        }
        .p(self.channel.to_string())
        .await;

        // Write to database
        db.clear_reaction(&self.id, emoji).await
    }
}

pub trait IntoUsers {
    fn get_user_ids(&self) -> Vec<String>;
}

impl IntoUsers for Message {
    fn get_user_ids(&self) -> Vec<String> {
        let mut ids = vec![self.author.clone()];

        if let Some(msg) = &self.system {
            match msg {
                SystemMessage::UserAdded { id, by, .. }
                | SystemMessage::UserRemove { id, by, .. } => {
                    ids.push(id.clone());
                    ids.push(by.clone());
                }
                SystemMessage::UserJoined { id, .. }
                | SystemMessage::UserLeft { id, .. }
                | SystemMessage::UserKicked { id, .. }
                | SystemMessage::UserBanned { id, .. } => ids.push(id.clone()),
                SystemMessage::ChannelRenamed { by, .. }
                | SystemMessage::ChannelDescriptionChanged { by, .. }
                | SystemMessage::ChannelIconChanged { by, .. } => ids.push(by.clone()),
                _ => {}
            }
        }

        ids
    }
}

impl IntoUsers for Vec<Message> {
    fn get_user_ids(&self) -> Vec<String> {
        let mut ids = vec![];
        for message in self {
            ids.append(&mut message.get_user_ids());
        }

        ids
    }
}

impl SystemMessage {
    pub fn into_message(self, channel: String) -> Message {
        Message {
            id: Ulid::new().to_string(),
            channel,
            author: "00000000000000000000000000".to_string(),
            system: Some(self),

            ..Default::default()
        }
    }
}

impl From<SystemMessage> for String {
    fn from(s: SystemMessage) -> String {
        match s {
            SystemMessage::Text { content } => content,
            SystemMessage::UserAdded { .. } => "User added to the channel.".to_string(),
            SystemMessage::UserRemove { .. } => "User removed from the channel.".to_string(),
            SystemMessage::UserJoined { .. } => "User joined the channel.".to_string(),
            SystemMessage::UserLeft { .. } => "User left the channel.".to_string(),
            SystemMessage::UserKicked { .. } => "User kicked from the channel.".to_string(),
            SystemMessage::UserBanned { .. } => "User banned from the channel.".to_string(),
            SystemMessage::ChannelRenamed { .. } => "Channel renamed.".to_string(),
            SystemMessage::ChannelDescriptionChanged { .. } => {
                "Channel description changed.".to_string()
            }
            SystemMessage::ChannelIconChanged { .. } => "Channel icon changed.".to_string(),
            SystemMessage::ChannelOwnershipChanged { .. } => {
                "Channel ownership changed.".to_string()
            }
        }
    }
}

impl SendableEmbed {
    pub async fn into_embed(self, db: &Database, message_id: String) -> Result<Embed> {
        self.validate()
            .map_err(|error| Error::FailedValidation { error })?;

        let media = if let Some(id) = self.media {
            Some(
                db.find_and_use_attachment(&id, "attachments", "message", &message_id)
                    .await?,
            )
        } else {
            None
        };

        Ok(Embed::Text(Text {
            icon_url: self.icon_url,
            url: self.url,
            title: self.title,
            description: self.description,
            media,
            colour: self.colour,
        }))
    }
}

impl BulkMessageResponse {
    pub async fn transform(
        db: &Database,
        channel: &Channel,
        messages: Vec<Message>,
        include_users: Option<bool>,
    ) -> Result<BulkMessageResponse> {
        if let Some(true) = include_users {
            let user_ids = messages.get_user_ids();
            let users = db.fetch_users(&user_ids).await?;

            Ok(match channel {
                Channel::TextChannel { server, .. } | Channel::VoiceChannel { server, .. } => {
                    BulkMessageResponse::MessagesAndUsers {
                        messages,
                        users,
                        members: Some(db.fetch_members(server, &user_ids).await?),
                    }
                }
                _ => BulkMessageResponse::MessagesAndUsers {
                    messages,
                    users,
                    members: None,
                },
            })
        } else {
            Ok(BulkMessageResponse::JustMessages(messages))
        }
    }
}

impl Interactions {
    /// Validate interactions info is correct
    pub async fn validate(&self, db: &Database) -> Result<()> {
        if let Some(reactions) = &self.reactions {
            if reactions.len() > 20 {
                return Err(Error::InvalidOperation);
            }

            for reaction in reactions {
                if !Emoji::can_use(db, reaction).await? {
                    return Err(Error::InvalidOperation);
                }
            }
        }

        Ok(())
    }

    /// Check if we can use a given emoji to react
    pub fn can_use(&self, emoji: &str) -> bool {
        if self.restrict_reactions {
            if let Some(reactions) = &self.reactions {
                reactions.contains(emoji)
            } else {
                false
            }
        } else {
            true
        }
    }

    /// Check if default initialisation of fields
    pub fn is_default(&self) -> bool {
        !self.restrict_reactions && self.reactions.is_none()
    }
}
