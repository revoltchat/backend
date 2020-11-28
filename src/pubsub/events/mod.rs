use serde::{Deserialize, Serialize};

pub mod groups;
pub mod guilds;
pub mod message;
pub mod users;

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Notification {
    message_create(message::Create),
    message_edit(message::Edit),
    message_delete(message::Delete),
    group_user_join(groups::UserJoin),
    group_user_leave(groups::UserLeave),
    guild_user_join(guilds::UserJoin),
    guild_user_leave(guilds::UserLeave),
    guild_channel_create(guilds::ChannelCreate),
    guild_channel_delete(guilds::ChannelDelete),
    guild_delete(guilds::Delete),
    user_friend_status(users::FriendStatus),
}

impl Notification {    
    pub fn push_to_cache(&self) {
        //crate::database::channel::process_event(&self);
        //crate::database::guild::process_event(&self);
        //crate::database::user::process_event(&self);
    }
}
