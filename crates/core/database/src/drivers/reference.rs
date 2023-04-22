use std::{collections::HashMap, sync::Arc};

use futures::lock::Mutex;

use crate::Bot;

database_derived!(
    /// Reference implementation
    #[derive(Default)]
    pub struct ReferenceDb {
        pub bots: Arc<Mutex<HashMap<String, Bot>>>,
    }
);
