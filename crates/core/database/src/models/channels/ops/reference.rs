#![allow(warnings)]
use super::AbstractChannels;
use crate::ReferenceDb;
use crate::{Channel, FieldsChannel, MongoDb, PartialChannel};
use futures::{FutureExt, StreamExt};
use revolt_permissions::OverrideField;
use revolt_result::Result;
static COL: &str = "channels";

#[async_trait]
impl AbstractChannels for ReferenceDb {
    /// Insert a new channel in the database
    async fn insert_channel(&self, channel: &Channel) -> Result<()> {
        let mut channels = self.channels.lock().await;
        if channels.contains_key(&channel.get_id()) {
            Err(create_database_error!("insert", "channel"))
        } else {
            channels.insert(channel.get_id(), channel.clone());
            Ok(())
        }
    }
    /// Insert a user to a group
    async fn add_user_to_group(&self, channel: &str, user: &str) -> Result<()> {
        let mut channels = self.channels.lock().await;

        if let Some(Channel::Group { recipients, .. }) = channels.get_mut(channel) {
            recipients.push(String::from(user));
            Ok(())
        } else {
            Err(create_error!(InvalidOperation))
        }
    }

    /// Insert channel role permissions
    async fn set_channel_role_permission(
        &self,
        channel: &str,
        role: &str,
        permissions: OverrideField,
    ) -> Result<()> {
        let mut channels = self.channels.lock().await;

        // find role id
        if let Some(mut channel) = channels.get_mut(channel) {
            match channel.clone() {
                Channel::TextChannel {
                    role_permissions, ..
                }
                | Channel::VoiceChannel {
                    role_permissions, ..
                } => {
                    let role = role_permissions.get(role);
                    if let Some(mut role_perm) = role {
                        role_perm = &permissions;
                    }

                    // todo create method for role editing
                    Ok(())
                }
                _ => Err(create_error!(InvalidOperation)),
            }
        } else {
            Err(create_error!(NotFound))
        }
    }

    /// Fetch a channel from the database
    async fn fetch_channel(&self, id: &str) -> Result<Channel> {
        todo!()
    }

    /// Fetch all channels from the database
    async fn fetch_channels<'a>(&self, ids: &'a [String]) -> Result<Vec<Channel>> {
        todo!()
    }

    /// Fetch all direct messages for a user
    async fn find_direct_messages(&self, user_id: &str) -> Result<Vec<Channel>> {
        todo!()
    }

    // Fetch saved messages channel
    async fn find_saved_messages_channel(&self, user_id: &str) -> Result<Channel> {
        todo!()
    }

    // Fetch direct message channel (PMs)
    async fn find_direct_message_channel(&self, user_a: &str, user_b: &str) -> Result<Channel> {
        todo!()
    }

    // Update channel
    async fn update_channel(
        &self,
        id: &str,
        channel: &Channel,
        remove: Vec<FieldsChannel>,
    ) -> Result<()> {
        todo!()
    }

    // Remove a user from a group
    async fn remove_user_from_group(&self, channel: &str, user: &str) -> Result<()> {
        todo!()
    }

    // Delete a channel
    async fn delete_channel(&self, channel: &Channel) -> Result<()> {
        todo!()
    }
}
