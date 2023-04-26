use std::{collections::HashMap, sync::Arc};

use futures::lock::Mutex;

use crate::{Bot, File, User};

database_derived!(
    /// Reference implementation
    #[derive(Default)]
    pub struct ReferenceDb {
        pub bots: Arc<Mutex<HashMap<String, Bot>>>,
        pub users: Arc<Mutex<HashMap<String, User>>>,
        pub files: Arc<Mutex<HashMap<String, File>>>,
    }
);
