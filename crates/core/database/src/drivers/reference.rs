use std::{collections::HashMap, sync::Arc};

use futures::lock::Mutex;

use crate::{
    AccountStrike, Bot, Channel, Event, File, Member, MemberCompositeKey, Server, User,
    UserSettings, UserWhiteList, Webhook,
};

database_derived!(
    /// Reference implementation
    #[derive(Default)]
    pub struct ReferenceDb {
        pub account_strikes: Arc<Mutex<HashMap<String, AccountStrike>>>,
        pub bots: Arc<Mutex<HashMap<String, Bot>>>,
        pub channel_webhooks: Arc<Mutex<HashMap<String, Webhook>>>,
        pub user_settings: Arc<Mutex<HashMap<String, UserSettings>>>,
        pub users: Arc<Mutex<HashMap<String, User>>>,
        pub server_members: Arc<Mutex<HashMap<MemberCompositeKey, Member>>>,
        pub servers: Arc<Mutex<HashMap<String, Server>>>,
        pub user_white_lists: Arc<Mutex<HashMap<String, UserWhiteList>>>,
        pub files: Arc<Mutex<HashMap<String, File>>>,
        pub server_bans: Arc<Mutex<HashMap<String, ()>>>,
        pub safety_reports: Arc<Mutex<HashMap<String, ()>>>,
        pub safety_snapshots: Arc<Mutex<HashMap<String, ()>>>,
        pub emoji: Arc<Mutex<HashMap<String, ()>>>,
        pub messages: Arc<Mutex<HashMap<String, ()>>>,
        pub channels: Arc<Mutex<HashMap<String, Channel>>>,
        pub channel_invites: Arc<Mutex<HashMap<String, ()>>>,
        pub channel_unreads: Arc<Mutex<HashMap<String, ()>>>,
        pub events: Arc<Mutex<HashMap<String, Event>>>,
    }
);
