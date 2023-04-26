use std::{collections::HashMap, sync::Arc};

use futures::lock::Mutex;

use crate::{Bot, File, Member, MemberCompositeKey, Server, User, UserSettings};

database_derived!(
    /// Reference implementation
    #[derive(Default)]
    pub struct ReferenceDb {
        pub bots: Arc<Mutex<HashMap<String, Bot>>>,
        pub user_settings: Arc<Mutex<HashMap<String, UserSettings>>>,
        pub users: Arc<Mutex<HashMap<String, User>>>,
        pub server_members: Arc<Mutex<HashMap<MemberCompositeKey, Member>>>,
        pub servers: Arc<Mutex<HashMap<String, Server>>>,
        pub files: Arc<Mutex<HashMap<String, File>>>,

        pub server_bans: Arc<Mutex<HashMap<String, ()>>>,
        pub safety_reports: Arc<Mutex<HashMap<String, ()>>>,
        pub safety_snapshots: Arc<Mutex<HashMap<String, ()>>>,
        pub emoji: Arc<Mutex<HashMap<String, ()>>>,
        pub messages: Arc<Mutex<HashMap<String, ()>>>,
        pub channels: Arc<Mutex<HashMap<String, ()>>>,
        pub channel_invites: Arc<Mutex<HashMap<String, ()>>>,
        pub channel_unreads: Arc<Mutex<HashMap<String, ()>>>,
    }
);
