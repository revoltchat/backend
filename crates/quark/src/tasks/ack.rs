// Queue Type: Debounced
use crate::Database;

use deadqueue::limited::Queue;
use mongodb::bson::doc;
use std::{collections::HashMap, time::Duration};

use super::DelayedTask;

/// Enumeration of possible events
#[derive(Debug, Eq, PartialEq)]
pub enum AckEvent {
    /// Add mentions for a user in a channel
    AddMention {
        /// Message IDs
        ids: Vec<String>,
    },

    /// Acknowledge message in a channel for a user
    AckMessage {
        /// Message ID
        id: String,
    },
}

/// Task information
struct Data {
    /// Channel to ack
    channel: String,
    /// User to ack for
    user: String,
    /// Event
    event: AckEvent,
}

#[derive(Debug)]
struct Task {
    event: AckEvent,
}

lazy_static! {
    static ref Q: Queue<Data> = Queue::new(10_000);
}

/// Queue a new task for a worker
pub async fn queue(channel: String, user: String, event: AckEvent) {
    Q.try_push(Data {
        channel,
        user,
        event,
    })
    .ok();

    info!("Queue is using {} slots from {}.", Q.len(), Q.capacity());
}

/// Start a new worker
pub async fn worker(db: Database) {
    let mut tasks = HashMap::<(String, String), DelayedTask<Task>>::new();
    let mut keys = vec![];

    loop {
        // Find due tasks.
        for (key, task) in &tasks {
            if task.should_run() {
                keys.push(key.clone());
            }
        }

        // Commit any due tasks to the database.
        for key in &keys {
            if let Some(task) = tasks.remove(key) {
                let Task { event } = task.data;
                let (user, channel) = key;

                if let Err(err) = match &event {
                    AckEvent::AckMessage { id } => db.acknowledge_message(channel, user, id).await,
                    AckEvent::AddMention { ids } => {
                        db.add_mention_to_unread(channel, user, ids).await
                    }
                } {
                    error!("{err:?} for {event:?}. ({user}, {channel})");
                } else {
                    info!("User {user} ack in {channel} with {event:?}");
                }
            }
        }

        // Clear keys
        keys.clear();

        // Queue incoming tasks.
        while let Some(Data {
            channel,
            user,
            mut event,
        }) = Q.try_pop()
        {
            let key = (user, channel);
            if let Some(mut task) = tasks.get_mut(&key) {
                task.delay();

                match &mut event {
                    AckEvent::AddMention { ids } => {
                        if let AckEvent::AddMention { ids: existing } = &mut task.data.event {
                            existing.append(ids);
                        } else {
                            task.data.event = event;
                        }
                    }
                    AckEvent::AckMessage { .. } => {
                        task.data.event = event;
                    }
                }
            } else {
                tasks.insert(key, DelayedTask::new(Task { event }));
            }
        }

        // Sleep for an arbitrary amount of time.
        async_std::task::sleep(Duration::from_secs(1)).await;
    }
}
