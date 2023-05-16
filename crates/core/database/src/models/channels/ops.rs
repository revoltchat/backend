use crate::{revolt_result::Result, Channel, FieldsChannel, PartialChannel};
use revolt_permissions::OverrideField;
mod mongodb;
mod reference;

#[async_trait]
pub trait AbstractChannels: Sync + Send {
    /// Insert a new channel in the database
    async fn insert_channel(&self, channel: &Channel) -> Result<()>;

    /// Insert a a user to a group
    async fn add_user_to_group(&self, channel: &str, user: &str) -> Result<()>;

    /// Insert channel role permissions
    async fn set_channel_role_permission(
        &self,
        channel: &str,
        role: &str,
        permissions: OverrideField,
    ) -> Result<()>;

    /// Fetch a channel from the database
    async fn fetch_channel(&self, id: &str) -> Result<Channel>;

    /// Fetch all channels from the database
    async fn fetch_channels<'a>(&self, ids: &'a [String]) -> Result<Vec<Channel>>;

    /// Fetch all direct messages for a user
    async fn find_direct_messages(&self, user_id: &str) -> Result<Vec<Channel>>;

    // Fetch saved messages channel
    async fn find_saved_messages_channel(&self, user_id: &str) -> Result<Channel>;

    // Fetch direct message channel (PMs)
    async fn find_direct_message_channel(&self, user_a: &str, user_b: &str) -> Result<Channel>;

    // Update channel
    async fn update_channel(
        &self,
        id: &str,
        channel: &PartialChannel,
        remove: Vec<FieldsChannel>,
    ) -> Result<()>;

    // Remove a user from a group
    async fn remove_user_from_group(&self, channel: &str, user: &str) -> Result<()>;

    // Delete a channel
    async fn delete_channel(&self, channel: &Channel) -> Result<()>;
}
