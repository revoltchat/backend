use std::collections::HashSet;

use crate::models::channel::{Channel, FieldsChannel, PartialChannel};
use crate::{OverrideField, Result};

#[async_trait]
pub trait AbstractChannel: Sync + Send {
    /// Fetch a channel by its id
    async fn fetch_channel(&self, id: &str) -> Result<Channel>;

    /// Fetch channels by their ids
    async fn fetch_channels<'a>(&self, ids: &'a [String]) -> Result<Vec<Channel>>;

    /// Insert a new channel into the database
    async fn insert_channel(&self, channel: &Channel) -> Result<()>;

    /// Update an existing channel using some data
    /// ! TODO: we need separate Channel::update which also sends out the relevant events
    /// ! also applies to other methods I guess, try to restrict event bound methods to
    /// ! the models themselves instead of the abstract database
    async fn update_channel(
        &self,
        id: &str,
        channel: &PartialChannel,
        remove: Vec<FieldsChannel>,
    ) -> Result<()>;

    /// Delete a channel by its id
    ///
    /// This will also delete all associated messages and files.
    async fn delete_channel(&self, channel: &Channel) -> Result<()>;

    /// Find all direct messages that a user is involved in
    ///
    /// Returns group DMs, any DMs marked as "active" and saved messages.
    async fn find_direct_messages(&self, user_id: &str) -> Result<Vec<Channel>>;

    /// Find a direct message channel between two users
    async fn find_direct_message_channel(&self, user_a: &str, user_b: &str) -> Result<Channel>;

    /// Find a saved message channel owned by a user
    async fn find_saved_messages_channel(&self, user_id: &str) -> Result<Channel>;

    /// Add user to a group
    async fn add_user_to_group(&self, channel: &str, user: &str) -> Result<()>;

    /// Remove a user from a group
    async fn remove_user_from_group(&self, channel: &str, user: &str) -> Result<()>;

    /// Set role permission for a channel
    /// ! FIXME: may want to refactor to just use normal updates
    async fn set_channel_role_permission(
        &self,
        channel: &str,
        role: &str,
        permissions: OverrideField,
    ) -> Result<()>;

    /// Validate existence of channels
    async fn check_channels_exist(&self, channels: &HashSet<String>) -> Result<bool>;
}
