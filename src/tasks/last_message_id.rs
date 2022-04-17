// Queue Type: Debounced
use deadqueue::limited::Queue;
use log::info;
use mongodb::bson::doc;
use revolt_quark::{models::channel::PartialChannel, Database};
use std::{collections::HashMap, time::Duration};

use super::DelayedTask;

struct Data {
    channel: String,
    id: String,
    is_dm: bool,
}

#[derive(Debug)]
struct Task {
    id: String,
    is_dm: bool,
}

lazy_static! {
    static ref Q: Queue<Data> = Queue::new(10_000);
}

pub async fn queue(channel: String, id: String, is_dm: bool) {
    Q.push(Data { channel, id, is_dm }).await;
}

pub async fn worker(db: Database) {
    let mut tasks = HashMap::<String, DelayedTask<Task>>::new();
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
                let Task { id, is_dm, .. } = task.data;

                let mut channel = PartialChannel {
                    last_message_id: Some(id.to_string()),
                    ..Default::default()
                };

                if is_dm {
                    channel.active = Some(true);
                }

                match db.update_channel(key, &channel, vec![]).await {
                    Ok(_) => info!("Updated last_message_id for {key} to {id}."),
                    Err(err) => error!("Failed to update last_message_id with {err:?}!"),
                }
            }
        }

        // Clear keys
        keys.clear();

        // Queue incoming tasks.
        while let Some(Data { channel, id, is_dm }) = Q.try_pop() {
            if let Some(mut task) = tasks.get_mut(&channel) {
                task.data.id = id;
                task.delay();
            } else {
                tasks.insert(channel, DelayedTask::new(Task { id, is_dm }));
            }
        }

        // Sleep for an arbitrary amount of time.
        async_std::task::sleep(Duration::from_secs(1)).await;
    }
}
