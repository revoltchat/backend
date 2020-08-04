use crate::database;
use crate::util::vec_to_set;

use mongodb::bson::doc;
use hashbrown::{HashMap, HashSet};
use mongodb::options::FindOneOptions;
use once_cell::sync::OnceCell;
use std::sync::RwLock;
use ws::Sender;

pub enum StateResult {
    DatabaseError,
    InvalidToken,
    Success(String),
}

static mut CONNECTIONS: OnceCell<RwLock<HashMap<String, Sender>>> = OnceCell::new();

pub fn add_connection(id: String, sender: Sender) {
    unsafe {
        CONNECTIONS
            .get()
            .unwrap()
            .write()
            .unwrap()
            .insert(id, sender);
    }
}

pub struct User {
    connections: HashSet<String>,
    guilds: HashSet<String>,
}

impl User {
    pub fn new() -> User {
        User {
            connections: HashSet::new(),
            guilds: HashSet::new(),
        }
    }
}

pub struct Guild {
    users: HashSet<String>,
}

impl Guild {
    pub fn new() -> Guild {
        Guild {
            users: HashSet::new(),
        }
    }
}

pub struct GlobalState {
    users: HashMap<String, User>,
    guilds: HashMap<String, Guild>,
}

impl GlobalState {
    pub fn new() -> GlobalState {
        GlobalState {
            users: HashMap::new(),
            guilds: HashMap::new(),
        }
    }

    pub fn push_to_guild(&mut self, guild: String, user: String) {
        if !self.guilds.contains_key(&guild) {
            self.guilds.insert(guild.clone(), Guild::new());
        }

        self.guilds.get_mut(&guild).unwrap().users.insert(user);
    }

    pub fn try_authenticate(&mut self, connection: String, access_token: String) -> StateResult {
        if let Ok(result) = database::get_collection("users").find_one(
            doc! {
                "access_token": access_token,
            },
            FindOneOptions::builder()
                .projection(doc! { "_id": 1 })
                .build(),
        ) {
            if let Some(user) = result {
                let user_id = user.get_str("_id").unwrap();

                if self.users.contains_key(user_id) {
                    self.users
                        .get_mut(user_id)
                        .unwrap()
                        .connections
                        .insert(connection);

                    return StateResult::Success(user_id.to_string());
                }

                if let Ok(results) =
                    database::get_collection("members").find(doc! { "_id.user": &user_id }, None)
                {
                    let mut guilds = vec![];
                    for result in results {
                        if let Ok(entry) = result {
                            guilds.push(
                                entry
                                    .get_document("_id")
                                    .unwrap()
                                    .get_str("guild")
                                    .unwrap()
                                    .to_string(),
                            );
                        }
                    }

                    let mut user = User::new();
                    for guild in guilds {
                        user.guilds.insert(guild.clone());
                        self.push_to_guild(guild, user_id.to_string());
                    }

                    user.connections.insert(connection);
                    self.users.insert(user_id.to_string(), user);

                    StateResult::Success(user_id.to_string())
                } else {
                    StateResult::DatabaseError
                }
            } else {
                StateResult::InvalidToken
            }
        } else {
            StateResult::DatabaseError
        }
    }

    pub fn disconnect<U: Into<Option<String>>>(&mut self, user_id: U, connection: String) {
        if let Some(user_id) = user_id.into() {
            let user = self.users.get_mut(&user_id).unwrap();
            user.connections.remove(&connection);

            if user.connections.len() == 0 {
                for guild in &user.guilds {
                    self.guilds.get_mut(guild).unwrap().users.remove(&user_id);
                }

                self.users.remove(&user_id);
            }
        }

        unsafe {
            CONNECTIONS
                .get()
                .unwrap()
                .write()
                .unwrap()
                .remove(&connection);
        }
    }
}

pub static mut DATA: OnceCell<RwLock<GlobalState>> = OnceCell::new();

pub fn init() {
    unsafe {
        if CONNECTIONS.set(RwLock::new(HashMap::new())).is_err() {
            panic!("Failed to set global connections map.");
        }

        if DATA.set(RwLock::new(GlobalState::new())).is_err() {
            panic!("Failed to set global state.");
        }
    }
}

pub fn send_message(users: Option<Vec<String>>, guild: Option<String>, data: String) {
    let state = unsafe { DATA.get().unwrap().read().unwrap() };
    let mut connections = HashSet::new();

    let mut users = vec_to_set(&users.unwrap_or(vec![]));
    if let Some(guild) = guild {
        if let Some(entry) = state.guilds.get(&guild) {
            for user in &entry.users {
                users.insert(user.to_string());
            }
        }
    }

    for user in users {
        if let Some(entry) = state.users.get(&user) {
            for connection in &entry.connections {
                connections.insert(connection.clone());
            }
        }
    }

    let targets = unsafe { CONNECTIONS.get().unwrap().read().unwrap() };
    for conn in connections {
        if let Some(sender) = targets.get(&conn) {
            if sender.send(data.clone()).is_err() {
                eprintln!("Failed to send a notification to a websocket. [{}]", &conn);
            }
        }
    }
}
