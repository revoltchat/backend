use super::AbstractChannels;
use crate::{Channel, FieldsChannel, MongoDb, PartialChannel};
use futures::StreamExt;
use revolt_permissions::OverrideField;
use revolt_result::Result;
static COL: &str = "channels";

#[async_trait]
impl AbstractChannels for MongoDb {
    async fn fetch_channel(&self, id: &str) -> Result<Channel> {
        query!(self, find_one_by_id, COL, id)?.ok_or_else(|| create_error!(NotFound))
    }
    async fn fetch_channels<'a>(&self, ids: &'a [String]) -> Result<Vec<Channel>> {
        Ok(self
            .col::<Channel>(COL)
            .find(
                doc! {
                    "_id": {
                        "$in": ids
                    }
                },
                None,
            )
            .await
            .map_err(|_| create_database_error!("find", "servers"))?
            .filter_map(|s| async {
                if cfg!(debug_assertions) {
                    Some(s.unwrap())
                } else {
                    s.ok()
                }
            })
            .collect()
            .await)
    }
    async fn insert_channel(&self, channel: &Channel) -> Result<()> {
        query!(self, insert_one, COL, &channel).map(|_| ())
    }
    async fn update_channel(
        &self,
        id: &str,
        partial: &PartialChannel,
        remove: Vec<FieldsChannel>,
    ) -> Result<()> {
        todo!()
    }
    async fn delete_channel(&self, channel: &Channel) -> Result<()> {
        let channel_id = match channel {
            Channel::SavedMessages { id, .. } => id,
            Channel::DirectMessage { id, .. } => id,
            Channel::Group { id, .. } => id,
            Channel::TextChannel { id, .. } => id,
            Channel::VoiceChannel { id, .. } => id,
        };
        query!(self, delete_one_by_id, COL, channel_id).map(|_| ())
    }
    async fn find_direct_messages(&self, user_id: &str) -> Result<Vec<Channel>> {
        todo!()
    }
    async fn find_saved_messages_channel(&self, user_id: &str) -> Result<Channel> {
        todo!()
    }
    async fn find_direct_message_channel(&self, user_a: &str, user_b: &str) -> Result<Channel> {
        todo!()
    }
    async fn add_user_to_group(&self, channel: &str, user: &str) -> Result<()> {
        todo!()
    }
    async fn remove_user_from_group(&self, channel: &str, user: &str) -> Result<()> {
        todo!()
    }
    async fn set_channel_role_permission(
        &self,
        channel: &str,
        role: &str,
        permissions: OverrideField,
    ) -> Result<()> {
        todo!()
    }
}
