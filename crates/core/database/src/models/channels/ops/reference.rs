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
        if channels.contains_key(&channel.id()) {
            Err(create_database_error!("insert", "channel"))
        } else {
            channels.insert(channel.id(), channel.clone());
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

        if let Some(mut channel) = channels.get_mut(channel) {
            // check for non override
            if let Some(mut role_data) = channel.find_role(role) {
                channel.set_role_permission(role, permissions).await
            } else {
                Err(create_error!(NotFound))
            }
        } else {
            Err(create_error!(NotFound))
        }
    }

    /// Fetch a channel from the database
    async fn fetch_channel(&self, id: &str) -> Result<Channel> {
        let mut channels = self.channels.lock().await;
        channels
            .get(id)
            .cloned()
            .ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch all channels from the database
    async fn fetch_channels<'a>(&self, ids: &'a [String]) -> Result<Vec<Channel>> {
        let mut channels = self.channels.lock().await;
        ids.iter()
            .map(|id| {
                channels
                    .get(id)
                    .cloned()
                    .ok_or_else(|| create_error!(NotFound))
            })
            .collect()
    }

    /// Fetch all direct messages for a user
    async fn find_direct_messages(&self, user_id: &str) -> Result<Vec<Channel>> {
        todo!()
    }

    // Fetch saved messages channel
    async fn find_saved_messages_channel(&self, user_id: &str) -> Result<Channel> {
        let mut channels = self.channels.lock().await;
        channels
            .get(user_id)
            .cloned()
            .ok_or_else(|| create_database_error!("fetch", "channel"))
    }

    // Fetch direct message channel (PMs)
    async fn find_direct_message_channel(&self, user_a: &str, user_b: &str) -> Result<Channel> {
        todo!()
    }

    // Update channel
    async fn update_channel(
        &self,
        id: &str,
        channel: &PartialChannel,
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
