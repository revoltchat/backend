use enum_dispatch::enum_dispatch;
use mongodb::{bson::Document, Client, Collection, Database as MoDatabase};
use once_cell::sync::OnceCell;
use web_push::SubscriptionInfo;

use drivers::{/*mockup::Mockup,*/mongo::MongoDB};
pub use entities::*;
pub use guards::*;
pub use permissions::*;

use crate::routes::channels::MsgSearchSort;
use crate::routes::servers::BannedUser;
use crate::util::{result::Result, variables::MONGO_URI};

static DBCONN: OnceCell<Database> = OnceCell::new();

pub async fn connect() {
    let dbconn = Database::new_from_mongo(&MONGO_URI).await;
    DBCONN.set(dbconn).unwrap();
}

pub fn db_conn() -> &'static Database {
    DBCONN.get().unwrap()
}

mod drivers;
pub mod entities;
pub mod guards;
pub mod permissions;

#[async_trait]
#[enum_dispatch]
pub trait Queries {
    // users
    async fn get_user_by_id(&self, id: &str) -> Result<User>;
    async fn get_user_by_username(&self, username: &str) -> Result<User>;
    async fn get_user_id_by_bot_token(&self, token: &str) -> Result<String>;
    async fn get_users(&self, user_ids: Vec<&str>) -> Result<Vec<User>>;
    async fn get_users_as_banned_users(&self, user_ids: Vec<&str>) -> Result<Vec<BannedUser>>;
    async fn get_bot_users_owned_by_user_id(&self, id: &str) -> Result<Vec<User>>;
    async fn get_mutual_friends_ids(&self, user_id_a: &str, user_id_b: &str)
                                    -> Result<Vec<String>>;
    async fn add_user(&self, id: &str, username: &str) -> Result<()>;
    async fn add_bot_user(&self, id: &str, username: &str, owner_id: &str) -> Result<()>;
    async fn delete_user(&self, id: &str) -> Result<()>;
    async fn update_username(&self, id: &str, new_username: &str) -> Result<()>;
    async fn make_user_already_in_relations_blocked(
        &self,
        origin_id: &str,
        target_id: &str,
    ) -> Result<()>;
    async fn make_user_already_in_relations_blocked_by(
        &self,
        target_id: &str,
        origin_id: &str,
    ) -> Result<()>;
    async fn make_user_not_in_relations_blocked(
        &self,
        origin_id: &str,
        target_id: &str,
    ) -> Result<()>;
    async fn make_user_not_in_relations_blocked_by(
        &self,
        target_id: &str,
        origin_id: &str,
    ) -> Result<()>;
    async fn apply_profile_changes(&self, id: &str, change_doc: Document) -> Result<()>;
    async fn remove_user_from_relations(&self, id: &str, target_id: &str) -> Result<()>;

    // accounts
    async fn get_accounts_subscriptions(
        &self,
        target_ids: Vec<&str>,
    ) -> Option<Vec<SubscriptionInfo>>;
    /*
    async fn subscribe(
        &self,
        account_id: &str,
        session_id: &str,
        subscription: Subscription,
    ) -> Result<()>;
     */
    async fn unsubscribe(&self, account_id: &str, session_id: &str) -> Result<()>;

    // attachments
    async fn get_attachment(&self, id: &str, tag: &str, parent_type: &str) -> Result<File>;
    async fn link_attachment_to_parent(
        &self,
        id: &str,
        parent_type: &str,
        parent_id: &str,
    ) -> Result<()>;
    async fn delete_attachment(&self, id: &str) -> Result<()>;
    async fn delete_attachments(&self, ids: Vec<&str>) -> Result<()>;
    async fn delete_attachments_of_messages(&self, message_ids: &Vec<String>) -> Result<()>;

    // bots
    async fn get_bot_count_owned_by_user(&self, user_id: &str) -> Result<u64>;
    async fn get_bots_owned_by_user_id(&self, id: &str) -> Result<Vec<Bot>>;
    async fn add_bot(&self, bot: &Bot) -> Result<()>;
    async fn delete_bot(&self, id: &str) -> Result<()>;
    async fn apply_bot_changes(&self, id: &str, change_doc: Document) -> Result<()>;

    // channel_invites
    async fn delete_invites_associated_to_channel(&self, id: &str) -> Result<()>;
    async fn delete_invites_associated_to_channels(&self, ids: &Vec<String>) -> Result<()>;
    async fn get_invite_by_id(&self, id: &str) -> Result<Invite>;
    async fn add_invite(&self, invite: &Invite) -> Result<()>;
    async fn delete_invite(&self, id: &str) -> Result<()>;
    async fn get_invites_of_server(&self, server_id: &str) -> Result<Vec<Invite>>;

    // channel_unreads
    async fn delete_channel_unreads(&self, channel_ids: &Vec<String>) -> Result<()>;
    async fn delete_multi_channel_unreads_for_user(
        &self,
        channel_ids: Vec<&str>,
        user_id: &str,
    ) -> Result<()>;
    async fn add_mentions_to_channel_unreads(
        &self,
        channel_id: &str,
        mentions: Vec<&str>,
        message: &str,
    ) -> Result<()>;
    async fn add_channels_to_unreads_for_user(
        &self,
        channel_ids: Vec<&str>,
        user_id: &str,
        current_time: &str,
    ) -> Result<()>;
    async fn get_unreads_for_user(&self, user_id: &str) -> Result<Vec<Document>>;
    async fn update_last_message_in_channel_unreads(
        &self,
        channel_id: &str,
        user_id: &str,
        message_id: &str,
    ) -> Result<()>;

    // channels
    async fn does_channel_exist_by_nonce(&self, nonce: &str) -> Result<bool>;
    async fn remove_recipient_from_channel(
        &self,
        channel_id: &str,
        recipient_id: &str,
    ) -> Result<()>;
    async fn update_channel_role_permissions(
        &self,
        channel_id: &str,
        role: &str,
        permissions: i32,
    ) -> Result<()>;
    async fn update_channel_permissions(&self, channel_id: &str, permissions: i32) -> Result<()>;
    async fn update_channel_default_permissions(
        &self,
        channel_id: &str,
        default_permissions: i32,
    ) -> Result<()>;
    async fn delete_server_channels_role_permissions(
        &self,
        server_id: &str,
        role_id: &str,
    ) -> Result<()>;
    async fn get_dm_channels_from_user(&self, user_id: &str) -> Result<Vec<Document>>;
    async fn get_dm_channel(&self, user_a: &str, user_b: &str) -> Result<Option<Document>>;
    async fn delete_all_channels_from_server(&self, server_id: &str) -> Result<()>;
    async fn add_channel(&self, channel: &Channel) -> Result<()>;
    async fn delete_channel(&self, id: &str) -> Result<()>;
    async fn add_recipient_to_channel(&self, channel_id: &str, recipient_id: &str) -> Result<()>;
    async fn are_users_connected_in_dms_or_group(&self, user_a: &str, user_b: &str)
                                                 -> Result<bool>;
    async fn get_sms_dms_groups_where_user_is_recipient(
        &self,
        channel_ids: Vec<&str>,
        user_id: &str,
    ) -> Result<Vec<Channel>>;
    async fn get_channel_ids_from_sms_dms_groups_where_user_is_recipient(
        &self,
        user_id: &str,
    ) -> Result<Vec<String>>;
    async fn make_channel_inactive(&self, channel_id: &str) -> Result<()>;
    async fn update_channel_owner(
        &self,
        channel_id: &str,
        new_owner: &str,
        old_owner: &str,
    ) -> Result<()>;
    async fn apply_channel_changes(&self, channel_id: &str, change_doc: Document) -> Result<()>;

    // messages
    async fn set_message_updates(&self, message_id: &str, set_doc: Document) -> Result<()>;
    async fn get_ids_from_messages_with_attachments(&self, channel_ids: &Vec<String>)
                                                    -> Result<Vec<String>>;
    async fn delete_messages_from_channels(&self, channel_ids: &Vec<String>) -> Result<()>;
    async fn add_message(&self, message: &Message) -> Result<()>;
    async fn add_embeds_to_message(&self, message_id: &str, embeds: &Vec<Embed>) -> Result<()>;
    async fn delete_message(&self, message_id: &str) -> Result<()>;
    async fn get_messages_by_ids_and_channel(
        &self,
        message_ids: Vec<&str>,
        channel_id: &str,
    ) -> Result<Vec<Message>>;
    async fn search_messages(
        &self,
        channel_id: &str,
        search: &str,
        options_before: Option<&str>,
        options_after: Option<&str>,
        limit: i64,
        sort: MsgSearchSort,
    ) -> Result<Vec<Message>>;
    async fn does_message_exist_by_nonce(&self, nonce: &str) -> Result<bool>;

    // server bans
    async fn delete_server_ban(&self, server_id: &str, user_id: &str) -> Result<()>;
    async fn is_user_banned(&self, server_id: &str, user_id: &str) -> Result<bool>;
    async fn get_ban(&self, server_id: &str, user_id: &str) -> Result<Ban>;
    async fn get_bans(&self, server_id: &str) -> Result<Vec<Ban>>;
    async fn add_server_ban(
        &self,
        server_id: &str,
        user_id: &str,
        reason: Option<&str>,
    ) -> Result<()>;

    // server members
    async fn get_server_member(&self, server_id: &str, user_id: &str) -> Result<Member>;
    async fn get_server_members(&self, server_id: &str) -> Result<Vec<Member>>;
    async fn add_server_member(&self, server_id: &str, user_id: &str) -> Result<()>;
    async fn delete_server_member(&self, server_id: &str, user_id: &str) -> Result<i64>;
    async fn get_server_member_count(&self, server_id: &str) -> Result<i64>;
    async fn get_users_memberships(&self, user_id: &str) -> Result<Vec<Member>>;
    async fn is_user_member_in_one_of_servers(
        &self,
        user_id: &str,
        server_ids: Vec<&str>,
    ) -> Result<bool>;
    async fn apply_server_member_changes(
        &self,
        server_id: &str,
        user_id: &str,
        change_doc: Document,
    ) -> Result<()>;
    async fn delete_role_from_server_members(&self, server_id: &str, role_id: &str) -> Result<()>;
    async fn get_server_memberships_by_ids(
        &self,
        user_id: &str,
        server_ids: Vec<&str>,
    ) -> Result<Vec<Member>>;

    // servers
    async fn update_server_permissions(
        &self,
        server_id: &str,
        role_id: &str,
        server_permissions: i32,
        channel_permissions: i32,
    ) -> Result<()>;
    async fn update_server_default_permissions(
        &self,
        server_id: &str,
        server_permissions: i32,
        channel_permissions: i32,
    ) -> Result<()>;
    async fn apply_server_changes(&self, server_id: &str, change_doc: Document) -> Result<()>;
    async fn add_server(&self, server: &Server) -> Result<()>;
    async fn get_servers(&self, server_ids: Vec<&str>) -> Result<Vec<Server>>;
    async fn add_channel_to_server(&self, server_id: &str, channel_id: &str) -> Result<()>;
    async fn create_role(
        &self,
        server_id: &str,
        role_id: &str,
        role_name: &str,
        default_permission: i32,
        default_permission_server: i32,
    ) -> Result<()>;
    async fn delete_role(&self, server_id: &str, role_id: &str) -> Result<()>;
    async fn does_server_exist_by_nonce(&self, nonce: &str) -> Result<bool>;

    // user settings
    async fn update_user_settings(&self, user_id: &str, set_doc: Document) -> Result<()>;
    async fn get_user_settings_doc(
        &self,
        user_id: &str,
        option_keys: Vec<&str>,
    ) -> Result<Option<Document>>;
}

#[enum_dispatch(Queries)]
pub enum Driver {
    Mongo(MongoDB),
    // Mockup(Mockup),
}

#[derive(Debug, Clone)]
pub struct Database {
    driver: Driver,
}

impl Database {
    pub async fn new_from_mongo(mongo_uri: &str) -> Self {
        Self {
            driver: Driver::from(MongoDB::new(mongo_uri).await),
        }
    }

    /*
    pub fn new_from_mockup() -> Self {
        let mockup = Mockup {};
        Self {
            driver: Driver::from(mockup),
        }
    }*/
}

#[async_trait]
impl Queries for Database {
    async fn get_user_by_id(&self, id: &str) -> Result<User> {
        self.driver.get_user_by_id(id).await
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User> {
        self.driver.get_user_by_username(username).await
    }

    async fn get_user_id_by_bot_token(&self, token: &str) -> Result<String> {
        self.driver.get_user_id_by_bot_token(token).await
    }

    async fn get_users(&self, user_ids: Vec<&str>) -> Result<Vec<User>> {
        self.driver.get_users(user_ids).await
    }

    async fn get_users_as_banned_users(&self, user_ids: Vec<&str>) -> Result<Vec<BannedUser>> {
        self.driver.get_users_as_banned_users(user_ids).await
    }

    async fn get_bot_users_owned_by_user_id(&self, id: &str) -> Result<Vec<User>> {
        self.driver.get_bot_users_owned_by_user_id(id).await
    }

    async fn get_mutual_friends_ids(
        &self,
        user_id_a: &str,
        user_id_b: &str,
    ) -> Result<Vec<String>> {
        self.driver
            .get_mutual_friends_ids(user_id_a, user_id_b)
            .await
    }

    async fn add_user(&self, id: &str, username: &str) -> Result<()> {
        self.driver.add_user(id, username).await
    }

    async fn add_bot_user(&self, id: &str, username: &str, owner_id: &str) -> Result<()> {
        self.driver.add_bot_user(id, username, owner_id).await
    }

    async fn delete_user(&self, id: &str) -> Result<()> {
        self.driver.delete_user(id).await
    }

    async fn update_username(&self, id: &str, new_username: &str) -> Result<()> {
        self.driver.update_username(id, new_username).await
    }

    async fn make_user_already_in_relations_blocked(
        &self,
        origin_id: &str,
        target_id: &str,
    ) -> Result<()> {
        self.make_user_already_in_relations_blocked(origin_id, target_id)
            .await
    }

    async fn make_user_already_in_relations_blocked_by(
        &self,
        target_id: &str,
        origin_id: &str,
    ) -> Result<()> {
        self.driver
            .make_user_already_in_relations_blocked_by(target_id, origin_id)
            .await
    }

    async fn make_user_not_in_relations_blocked(
        &self,
        origin_id: &str,
        target_id: &str,
    ) -> Result<()> {
        self.driver
            .make_user_not_in_relations_blocked(origin_id, target_id)
            .await
    }

    async fn make_user_not_in_relations_blocked_by(
        &self,
        target_id: &str,
        origin_id: &str,
    ) -> Result<()> {
        self.driver
            .make_user_not_in_relations_blocked_by(target_id, origin_id)
            .await
    }

    async fn apply_profile_changes(&self, id: &str, change_doc: Document) -> Result<()> {
        self.driver.apply_profile_changes(id, change_doc).await
    }

    async fn remove_user_from_relations(&self, id: &str, target: &str) -> Result<()> {
        self.driver.remove_user_from_relations(id, target).await
    }

    async fn get_accounts_subscriptions(
        &self,
        target_ids: Vec<&str>,
    ) -> Option<Vec<SubscriptionInfo>> {
        self.driver.get_accounts_subscriptions(target_ids).await
    }

    /*
    async fn subscribe(
        &self,
        account_id: &str,
        session_id: &str,
        subscription: Subscription,
    ) -> Result<()> {
        self.driver
            .subscribe(account_id, session_id, subscription)
            .await
    }*/

    async fn unsubscribe(&self, account_id: &str, session_id: &str) -> Result<()> {
        self.driver.unsubscribe(account_id, session_id).await
    }

    async fn get_attachment(&self, id: &str, tag: &str, parent_type: &str) -> Result<File> {
        self.driver.get_attachment(id, tag, parent_type).await
    }

    async fn link_attachment_to_parent(
        &self,
        id: &str,
        parent_type: &str,
        parent_id: &str,
    ) -> Result<()> {
        self.driver
            .link_attachment_to_parent(id, parent_type, parent_id)
            .await
    }

    async fn delete_attachment(&self, id: &str) -> Result<()> {
        self.driver.delete_attachment(id).await
    }

    async fn delete_attachments(&self, ids: Vec<&str>) -> Result<()> {
        self.driver.delete_attachments(ids).await
    }

    async fn delete_attachments_of_messages(&self, message_ids: &Vec<String>) -> Result<()> {
        self.driver
            .delete_attachments_of_messages(message_ids)
            .await
    }

    async fn get_bot_count_owned_by_user(&self, user_id: &str) -> Result<u64> {
        self.driver.get_bot_count_owned_by_user(user_id).await
    }

    async fn get_bots_owned_by_user_id(&self, id: &str) -> Result<Vec<Bot>> {
        self.driver.get_bots_owned_by_user_id(id).await
    }

    async fn add_bot(&self, bot: &Bot) -> Result<()> {
        self.driver.add_bot(bot).await
    }

    async fn delete_bot(&self, id: &str) -> Result<()> {
        self.delete_bot(id).await
    }

    async fn apply_bot_changes(&self, id: &str, change_doc: Document) -> Result<()> {
        self.driver.apply_bot_changes(id, change_doc).await
    }

    async fn delete_invites_associated_to_channel(&self, id: &str) -> Result<()> {
        self.driver.delete_invites_associated_to_channel(id).await
    }

    async fn delete_invites_associated_to_channels(&self, ids: &Vec<String>) -> Result<()> {
        self.driver.delete_invites_associated_to_channels(ids).await
    }

    async fn get_invite_by_id(&self, id: &str) -> Result<Invite> {
        self.driver.get_invite_by_id(id).await
    }

    async fn add_invite(&self, invite: &Invite) -> Result<()> {
        self.driver.add_invite(invite).await
    }

    async fn delete_invite(&self, id: &str) -> Result<()> {
        self.driver.delete_invite(id).await
    }

    async fn get_invites_of_server(&self, server_id: &str) -> Result<Vec<Invite>> {
        self.driver.get_invites_of_server(server_id).await
    }

    async fn delete_channel_unreads(&self, channel_ids: &Vec<String>) -> Result<()> {
        self.driver.delete_channel_unreads(channel_ids).await
    }

    async fn delete_multi_channel_unreads_for_user(
        &self,
        channel_ids: Vec<&str>,
        user_id: &str,
    ) -> Result<()> {
        self.driver
            .delete_multi_channel_unreads_for_user(channel_ids, user_id)
            .await
    }

    async fn add_mentions_to_channel_unreads(
        &self,
        channel_id: &str,
        mentions: Vec<&str>,
        message: &str,
    ) -> Result<()> {
        self.driver
            .add_mentions_to_channel_unreads(channel_id, mentions, message)
            .await
    }

    async fn add_channels_to_unreads_for_user(
        &self,
        channel_ids: Vec<&str>,
        user_id: &str,
        current_time: &str,
    ) -> Result<()> {
        self.driver
            .add_channels_to_unreads_for_user(channel_ids, user_id, current_time)
            .await
    }

    async fn get_unreads_for_user(&self, user_id: &str) -> Result<Vec<Document>> {
        self.driver.get_unreads_for_user(user_id).await
    }

    async fn update_last_message_in_channel_unreads(
        &self,
        channel_id: &str,
        user_id: &str,
        message_id: &str,
    ) -> Result<()> {
        self.driver
            .update_last_message_in_channel_unreads(channel_id, user_id, message_id)
            .await
    }

    async fn does_channel_exist_by_nonce(&self, nonce: &str) -> Result<bool> {
        self.driver.does_channel_exist_by_nonce(nonce).await
    }

    async fn remove_recipient_from_channel(
        &self,
        channel_id: &str,
        recipient_id: &str,
    ) -> Result<()> {
        self.driver
            .remove_recipient_from_channel(channel_id, recipient_id)
            .await
    }

    async fn update_channel_role_permissions(
        &self,
        channel_id: &str,
        role: &str,
        permissions: i32,
    ) -> Result<()> {
        self.driver
            .update_channel_role_permissions(channel_id, role, permissions)
            .await
    }

    async fn update_channel_permissions(&self, channel_id: &str, permissions: i32) -> Result<()> {
        self.driver
            .update_channel_permissions(channel_id, permissions)
            .await
    }

    async fn update_channel_default_permissions(
        &self,
        channel_id: &str,
        default_permissions: i32,
    ) -> Result<()> {
        self.driver
            .update_channel_default_permissions(channel_id, default_permissions)
            .await
    }

    async fn delete_server_channels_role_permissions(
        &self,
        server_id: &str,
        role_id: &str,
    ) -> Result<()> {
        self.driver
            .delete_server_channels_role_permissions(server_id, role_id)
            .await
    }

    async fn get_dm_channels_from_user(&self, user_id: &str) -> Result<Vec<Document>> {
        self.driver.get_dm_channels_from_user(user_id).await
    }

    async fn get_dm_channel(&self, user_a: &str, user_b: &str) -> Result<Option<Document>> {
        self.driver.get_dm_channel(user_a, user_b).await
    }

    async fn delete_all_channels_from_server(&self, server_id: &str) -> Result<()> {
        self.driver.delete_all_channels_from_server(server_id).await
    }

    async fn add_channel(&self, channel: &Channel) -> Result<()> {
        self.driver.add_channel(channel).await
    }

    async fn delete_channel(&self, id: &str) -> Result<()> {
        self.driver.delete_channel(id).await
    }

    async fn add_recipient_to_channel(&self, channel_id: &str, recipient_id: &str) -> Result<()> {
        self.driver
            .add_recipient_to_channel(channel_id, recipient_id)
            .await
    }

    async fn are_users_connected_in_dms_or_group(
        &self,
        user_a: &str,
        user_b: &str,
    ) -> Result<bool> {
        self.driver
            .are_users_connected_in_dms_or_group(user_a, user_b)
            .await
    }

    async fn get_sms_dms_groups_where_user_is_recipient(
        &self,
        channel_ids: Vec<&str>,
        user_id: &str,
    ) -> Result<Vec<Channel>> {
        self.driver
            .get_sms_dms_groups_where_user_is_recipient(channel_ids, user_id)
            .await
    }

    async fn get_channel_ids_from_sms_dms_groups_where_user_is_recipient(
        &self,
        user_id: &str,
    ) -> Result<Vec<String>> {
        self.driver
            .get_channel_ids_from_sms_dms_groups_where_user_is_recipient(user_id)
            .await
    }

    async fn make_channel_inactive(&self, channel_id: &str) -> Result<()> {
        self.driver.make_channel_inactive(channel_id).await
    }

    async fn update_channel_owner(
        &self,
        channel_id: &str,
        new_owner: &str,
        old_owner: &str,
    ) -> Result<()> {
        self.driver
            .update_channel_owner(channel_id, new_owner, old_owner)
            .await
    }

    async fn apply_channel_changes(&self, channel_id: &str, change_doc: Document) -> Result<()> {
        self.driver
            .apply_channel_changes(channel_id, change_doc)
            .await
    }

    async fn set_message_updates(&self, message_id: &str, set_doc: Document) -> Result<()> {
        self.driver.set_message_updates(message_id, set_doc).await
    }

    async fn get_ids_from_messages_with_attachments(
        &self,
        channel_ids: &Vec<String>,
    ) -> Result<Vec<String>> {
        self.driver
            .get_ids_from_messages_with_attachments(channel_ids)
            .await
    }

    async fn delete_messages_from_channels(&self, channel_ids: &Vec<String>) -> Result<()> {
        self.driver.delete_messages_from_channel(channel_ids).await
    }

    async fn add_message(&self, message: &Message) -> Result<()> {
        self.driver.add_message(message).await
    }

    async fn add_embeds_to_message(&self, message_id: &str, embeds: &Vec<Embed>) -> Result<()> {
        self.driver.add_embeds_to_message(message_id, embeds).await
    }

    async fn delete_message(&self, message_id: &str) -> Result<()> {
        self.driver.delete_message(message_id).await
    }

    async fn get_messages_by_ids_and_channel(
        &self,
        message_ids: Vec<&str>,
        channel_id: &str,
    ) -> Result<Vec<Message>> {
        self.driver
            .get_messages_by_ids_and_channel(message_ids, channel_id)
            .await
    }

    async fn search_messages(
        &self,
        channel_id: &str,
        search: &str,
        options_before: Option<&str>,
        options_after: Option<&str>,
        limit: i64,
        sort: MsgSearchSort,
    ) -> Result<Vec<Message>> {
        self.driver
            .search_messages(
                channel_id,
                search,
                options_before,
                options_after,
                limit,
                sort,
            )
            .await
    }

    async fn does_message_exist_by_nonce(&self, nonce: &str) -> Result<bool> {
        self.driver.does_message_exist_by_nonce(nonce).await
    }

    async fn delete_server_ban(&self, server_id: &str, user_id: &str) -> Result<()> {
        self.driver.delete_server_ban(server_id, user_id).await
    }

    async fn is_user_banned(&self, server_id: &str, user_id: &str) -> Result<bool> {
        self.driver.is_user_banned(server_id, user_id).await
    }

    async fn get_ban(&self, server_id: &str, user_id: &str) -> Result<Ban> {
        self.driver.get_ban(server_id, user_id).await
    }

    async fn get_bans(&self, server_id: &str) -> Result<Vec<Ban>> {
        self.driver.get_bans(server_id).await
    }

    async fn add_server_ban(
        &self,
        server_id: &str,
        user_id: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        self.driver.add_server_ban(server_id, user_id, reason).await
    }

    async fn get_server_member(&self, server_id: &str, user_id: &str) -> Result<Member> {
        self.driver.get_server_member(server_id, user_id).await
    }

    async fn get_server_members(&self, server_id: &str) -> Result<Vec<Member>> {
        self.driver.get_server_members(server_id).await
    }

    async fn add_server_member(&self, server_id: &str, user_id: &str) -> Result<()> {
        self.driver.add_server_member(server_id, user_id).await
    }

    async fn delete_server_member(&self, server_id: &str, user_id: &str) -> Result<i64> {
        self.driver.delete_server_member(server_id, user_id).await
    }

    async fn get_server_member_count(&self, server_id: &str) -> Result<i64> {
        self.driver.get_server_member_count(server_id).await
    }

    async fn get_users_memberships(&self, user_id: &str) -> Result<Vec<Member>> {
        self.driver.get_users_memberships(user_id).await
    }

    async fn is_user_member_in_one_of_servers(
        &self,
        user_id: &str,
        server_ids: Vec<&str>,
    ) -> Result<bool> {
        self.driver
            .is_user_member_in_one_of_servers(user_id, server_ids)
            .await
    }

    async fn apply_server_member_changes(
        &self,
        server_id: &str,
        user_id: &str,
        change_doc: Document,
    ) -> Result<()> {
        self.driver
            .apply_server_member_changes(server_id, user_id, change_doc)
            .await
    }

    async fn delete_role_from_server_members(&self, server_id: &str, role_id: &str) -> Result<()> {
        self.driver
            .delete_role_from_server_members(server_id, role_id)
            .await
    }

    async fn get_server_memberships_by_ids(
        &self,
        user_id: &str,
        server_ids: Vec<&str>,
    ) -> Result<Vec<Member>> {
        self.driver
            .get_server_memberships_by_ids(user_id, server_ids)
            .await
    }

    async fn update_server_permissions(
        &self,
        server_id: &str,
        role_id: &str,
        server_permissions: i32,
        channel_permissions: i32,
    ) -> Result<()> {
        self.driver
            .update_server_permissions(server_id, role_id, server_permissions, channel_permissions)
            .await
    }

    async fn update_server_default_permissions(
        &self,
        server_id: &str,
        server_permissions: i32,
        channel_permissions: i32,
    ) -> Result<()> {
        self.driver
            .update_server_default_permissions(server_id, server_permissions, channel_permissions)
            .await
    }

    async fn apply_server_changes(&self, server_id: &str, change_doc: Document) -> Result<()> {
        self.driver
            .apply_server_changes(server_id, change_doc)
            .await
    }

    async fn add_server(&self, server: &Server) -> Result<()> {
        self.driver.add_server(server).await
    }

    async fn get_servers(&self, server_ids: Vec<&str>) -> Result<Vec<Server>> {
        self.driver.get_servers(server_ids).await
    }

    async fn add_channel_to_server(&self, server_id: &str, channel_id: &str) -> Result<()> {
        self.driver
            .add_channel_to_server(server_id, channel_id)
            .await
    }

    async fn create_role(
        &self,
        server_id: &str,
        role_id: &str,
        role_name: &str,
        default_permission: i32,
        default_permission_server: i32,
    ) -> Result<()> {
        self.driver
            .create_role(
                server_id,
                role_id,
                role_name,
                default_permission,
                default_permission_server,
            )
            .await
    }

    async fn delete_role(&self, server_id: &str, role_id: &str) -> Result<()> {
        self.driver.delete_role(server_id, role_id).await
    }

    async fn does_server_exist_by_nonce(&self, nonce: &str) -> Result<bool> {
        self.driver.does_server_exist_by_nonce(nonce).await
    }

    async fn update_user_settings(&self, user_id: &str, set_doc: Document) -> Result<()> {
        self.driver.update_user_settings(user_id, set_doc).await
    }

    async fn get_user_settings_doc(
        &self,
        user_id: &str,
        option_keys: Vec<&str>,
    ) -> Result<Option<Document>> {
        self.driver
            .get_user_settings_doc(user_id, option_keys)
            .await
    }
}
