use std::collections::HashSet;

use crate::models::channel::{Channel, FieldsChannel, PartialChannel};
use crate::{AbstractAttachment, AbstractChannel, Error, OverrideField, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractChannel for DummyDb {
    async fn fetch_channel(&self, id: &str) -> Result<Channel> {
        Ok(Channel::Group {
            id: id.into(),

            name: "group".into(),
            owner: "owner".into(),
            description: None,
            recipients: vec!["owner".into()],

            icon: Some(
                self.find_and_use_attachment("dummy", "dummy", "dummy", "dummy")
                    .await?,
            ),
            last_message_id: None,

            permissions: None,

            nsfw: false,
        })
    }

    async fn fetch_channels<'a>(&self, _ids: &'a [String]) -> Result<Vec<Channel>> {
        Ok(vec![self.fetch_channel("sus").await.unwrap()])
    }

    async fn insert_channel(&self, channel: &Channel) -> Result<()> {
        info!("Insert {channel:?}");
        Ok(())
    }

    async fn update_channel(
        &self,
        id: &str,
        channel: &PartialChannel,
        remove: Vec<FieldsChannel>,
    ) -> Result<()> {
        info!("Update {id} with {channel:?} and remove {remove:?}");
        Ok(())
    }

    async fn delete_channel(&self, channel: &Channel) -> Result<()> {
        info!("Delete {channel:?}");
        Ok(())
    }

    async fn find_direct_messages(&self, user_id: &str) -> Result<Vec<Channel>> {
        Ok(vec![self.fetch_channel(user_id).await?])
    }

    async fn find_saved_messages_channel(&self, user: &str) -> Result<Channel> {
        self.fetch_channel(user).await
    }

    async fn find_direct_message_channel(&self, _user_a: &str, _user_b: &str) -> Result<Channel> {
        Err(Error::NotFound)
    }

    async fn add_user_to_group(&self, channel: &str, user: &str) -> Result<()> {
        info!("Added {user} to {channel}");
        Ok(())
    }

    async fn remove_user_from_group(&self, channel: &str, user: &str) -> Result<()> {
        info!("Removed {user} from {channel}");
        Ok(())
    }

    async fn set_channel_role_permission(
        &self,
        channel: &str,
        role: &str,
        permissions: OverrideField,
    ) -> Result<()> {
        info!("Updating permissions for role {role} in {channel} with {permissions:?}");
        Ok(())
    }

    async fn check_channels_exist(&self, _channels: &HashSet<String>) -> Result<bool> {
        Ok(true)
    }
}
