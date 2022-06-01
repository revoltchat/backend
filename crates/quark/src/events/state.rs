use std::collections::{HashMap, HashSet};

use crate::models::{Channel, Member, Server, User};

/// Enumeration representing some change in subscriptions
pub enum SubscriptionStateChange {
    /// No change
    None,
    /// Clear all subscriptions
    Reset,
    /// Append or remove subscriptions
    Change {
        add: Vec<String>,
        remove: Vec<String>,
    },
}

/// Dumb per-state cache implementation
///
/// Ideally this would use a global cache that
/// allows for mutations and could use Rc<> to
/// track usage. If Rc<> == 1, then it only
/// remains in global cache, hence should be
/// dropped.
///
/// ------------------------------------------------
/// We can strip these objects to core information!!
/// ------------------------------------------------
#[derive(Debug, Default)]
pub struct Cache {
    pub user_id: String,

    pub users: HashMap<String, User>,
    pub channels: HashMap<String, Channel>,
    pub members: HashMap<String, Member>,
    pub servers: HashMap<String, Server>,
}

/// Client state
pub struct State {
    pub cache: Cache,

    pub private_topic: String,
    subscribed: HashSet<String>,
    state: SubscriptionStateChange,
}

impl State {
    /// Create state from User
    pub fn from(user: User) -> State {
        let mut subscribed = HashSet::new();
        let private_topic = format!("{}!", user.id);
        subscribed.insert(private_topic.clone());
        subscribed.insert(user.id.clone());

        let mut cache: Cache = Cache {
            user_id: user.id.clone(),
            ..Default::default()
        };

        cache.users.insert(user.id.clone(), user);

        State {
            cache,
            subscribed,
            private_topic,
            state: SubscriptionStateChange::Reset,
        }
    }

    /// Apply currently queued state
    pub fn apply_state(&mut self) -> SubscriptionStateChange {
        let state = std::mem::replace(&mut self.state, SubscriptionStateChange::None);
        if let SubscriptionStateChange::Change { add, remove } = &state {
            for id in add {
                self.subscribed.insert(id.clone());
            }

            for id in remove {
                self.subscribed.remove(id);
            }
        }

        state
    }

    /// Clone the active user
    pub fn clone_user(&self) -> User {
        self.cache.users.get(&self.cache.user_id).unwrap().clone()
    }

    /// Iterate through all subscriptions
    pub fn iter_subscriptions(&self) -> std::collections::hash_set::Iter<'_, std::string::String> {
        self.subscribed.iter()
    }

    /// Reset the current state
    pub fn reset_state(&mut self) {
        self.state = SubscriptionStateChange::Reset;
        self.subscribed.clear();
    }

    /// Add a new subscription
    pub fn insert_subscription(&mut self, subscription: String) {
        if self.subscribed.contains(&subscription) {
            return;
        }

        match &mut self.state {
            SubscriptionStateChange::None => {
                self.state = SubscriptionStateChange::Change {
                    add: vec![subscription.clone()],
                    remove: vec![],
                };
            }
            SubscriptionStateChange::Change { add, .. } => {
                add.push(subscription.clone());
            }
            SubscriptionStateChange::Reset => {}
        }

        self.subscribed.insert(subscription);
    }

    /// Remove existing subscription
    pub fn remove_subscription(&mut self, subscription: &str) {
        if !self.subscribed.contains(&subscription.to_string()) {
            return;
        }

        match &mut self.state {
            SubscriptionStateChange::None => {
                self.state = SubscriptionStateChange::Change {
                    add: vec![],
                    remove: vec![subscription.to_string()],
                };
            }
            SubscriptionStateChange::Change { remove, .. } => {
                remove.push(subscription.to_string());
            }
            SubscriptionStateChange::Reset => panic!("Should not remove during a reset!"),
        }

        self.subscribed.remove(subscription);
    }
}
