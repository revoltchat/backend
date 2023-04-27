use crate::{revolt_result::Result, Channel, FieldsChannel, PartialChannel};
use revolt_permissions::OverrideField;
mod mongodb;
mod reference;

#[async_trait]
pub trait AbstractChannels: Sync + Send {
    async fn fetch_channel(&self, id: &str) -> Result<Channel>;
    async fn fetch_channels<'a>(&self, ids: &'a [String]) -> Result<Vec<Channel>>;
    async fn insert_channel(&self, channel: &Channel) -> Result<()>;
    async fn update_channel(
        &self,
        id: &str,
        channel: &PartialChannel,
        remove: Vec<FieldsChannel>,
    ) -> Result<()>;
    async fn delete_channel(&self, channel: &Channel) -> Result<()>;
    async fn find_direct_messages(&self, user_id: &str) -> Result<Vec<Channel>>;
    async fn find_saved_messages_channel(&self, user_id: &str) -> Result<Channel>;
    async fn find_direct_message_channel(&self, user_a: &str, user_b: &str) -> Result<Channel>;
    async fn add_user_to_group(&self, channel: &str, user: &str) -> Result<()>;
    async fn remove_user_from_group(&self, channel: &str, user: &str) -> Result<()>;
    async fn set_channel_role_permission(
        &self,
        channel: &str,
        role: &str,
        permissions: OverrideField,
    ) -> Result<()>;
}
