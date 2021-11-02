// Queue Type: Debounced
// TODO: Group similar events together with $in.

use std::{collections::HashMap, time::{Duration, Instant}};

use async_channel::{ Sender, Receiver, bounded };
use log::info;
use mongodb::{bson::doc, options::UpdateOptions};

use crate::database::*;

// Commit to database every 30 seconds if the task is particularly active.
static EXPIRE_CONSTANT: u64 = 30;

// Otherwise, commit to database after 5 seconds.
static SAVE_CONSTANT: u64 = 5;

struct Message {
    channel: String,
    user: String,
    event: AckEvent,
}

#[derive(Debug, Eq, PartialEq)]
pub enum AckEvent {
    AddMention { ids: Vec<String> },
    AckMessage { id: String }
}

#[derive(Debug)]
struct Task {
    event: AckEvent,
    last_updated: Instant,
    first_seen: Instant,
}

lazy_static! {
    static ref CHANNEL: (Sender<Message>, Receiver<Message>) = bounded(100);
}

pub async fn queue(channel: String, user: String, event: AckEvent) {
    CHANNEL.0.send(Message { channel, user, event }).await.ok();
}

pub async fn run() {
    let unreads = get_collection("channel_unreads");
    let mut tasks = HashMap::<(String, String), Task>::new();
    let mut keys = vec![];

    loop {
        // Find due tasks.
        for (key, Task { first_seen, last_updated, .. }) in &tasks {
            if first_seen.elapsed().as_secs() > EXPIRE_CONSTANT ||
                last_updated.elapsed().as_secs() > SAVE_CONSTANT {
                keys.push(key.clone());
            }
        }

        // Commit any due tasks to the database.
        for key in &keys {
            if let Some(Task { event, .. }) = tasks.remove(key) {
                let (user, channel) = key;

                match event {
                    AckEvent::AddMention { ids } => {
                        unreads.update_one(
                            doc! {
                                "_id.channel": channel,
                                "_id.user": user,
                            },
                            doc! {
                                "$push": {
                                    "mentions": {
                                        "$each": ids
                                    }
                                }
                            },
                            UpdateOptions::builder().upsert(true).build(),
                        )
                        .await
                        .ok();

                        info!("Added mentions for {} in {}.", user, channel);
                    },
                    AckEvent::AckMessage { id } => {
                        unreads.update_one(
                            doc! {
                                "_id.channel": channel,
                                "_id.user": user,
                            },
                            doc! {
                                "$unset": {
                                    "mentions": 1
                                },
                                "$set": {
                                    "last_id": id
                                }
                            },
                            UpdateOptions::builder().upsert(true).build(),
                        )
                        .await
                        .ok();

                        info!("User {} acknowledged {}.", user, channel);
                    }
                }
            }
        }

        // Clear keys
        keys.clear();

        // Queue incoming tasks.
        while let Ok(Message { channel, user, mut event }) = CHANNEL.1.try_recv() {
            let key = (user, channel);
            if let Some(mut existing_task) = tasks.get_mut(&key) {
                existing_task.last_updated = Instant::now();

                match &mut event {
                    AckEvent::AddMention { ids } => {
                        if let AckEvent::AddMention { ids: existing } = &mut existing_task.event {
                            existing.append(ids);
                        } else {
                            existing_task.event = event;
                        }
                    }
                    AckEvent::AckMessage { .. } => {
                        existing_task.event = event;
                    }
                }
            } else {
                tasks.insert(key, Task {
                    event,
                    last_updated: Instant::now(),
                    first_seen: Instant::now()
                });
            }
        }

        // Sleep for an arbitrary amount of time.
        async_std::task::sleep(Duration::from_secs(1)).await;
    }
}
