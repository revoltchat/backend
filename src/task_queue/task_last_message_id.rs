// Queue Type: Debounced
use std::{collections::HashMap, time::{Duration, Instant}};

use async_channel::{ Sender, Receiver, bounded };
use mongodb::bson::doc;

use crate::database::*;

// Commit to database every 30 seconds if the task is particularly active.
static EXPIRE_CONSTANT: u64 = 30;

// Otherwise, commit to database after 5 seconds.
static SAVE_CONSTANT: u64 = 5;

type Message = (String, String, bool);

#[derive(Debug)]
struct Task {
    id: String,
    is_dm: bool,
    last_updated: Instant,
    first_seen: Instant,
}

lazy_static! {
    static ref CHANNEL: (Sender<Message>, Receiver<Message>) = bounded(100);
}

pub async fn queue(channel: String, id: String, is_dm: bool) {
    CHANNEL.0.send((channel, id, is_dm)).await.ok();
}

pub async fn run() {
    let channels = get_collection("channels");
    let mut tasks = HashMap::<String, Task>::new();
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
            if let Some(Task { id, is_dm, .. }) = tasks.remove(key) {
                let mut set = doc! { "last_message_id": id.clone() };

                if is_dm {
                    set.insert("active", true);
                }

                channels
                    .update_one(
                        doc! { "_id": key },
                        doc! { "$set": set },
                        None,
                    )
                    .await
                    .ok();
            }
        }

        // Clear keys
        keys.clear();

        // Queue incoming tasks.
        while let Ok((channel, id, is_dm)) = CHANNEL.1.try_recv() {
            if let Some(mut existing_task) = tasks.get_mut(&channel) {
                existing_task.id = id;
                existing_task.last_updated = Instant::now();
            } else {
                tasks.insert(channel, Task {
                    id,
                    is_dm,
                    last_updated: Instant::now(),
                    first_seen: Instant::now()
                });
            }
        }

        // Sleep for an arbitrary amount of time.
        async_std::task::sleep(Duration::from_secs(1)).await;
    }
}
