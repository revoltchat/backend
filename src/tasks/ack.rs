// Queue Type: Debounced
use deadqueue::limited::Queue;
use mongodb::bson::doc;
use revolt_quark::Database;
use std::{collections::HashMap, time::Duration};

use super::DelayedTask;

#[derive(Debug, Eq, PartialEq)]
pub enum AckEvent {
    AddMention { ids: Vec<String> },
    AckMessage { id: String },
}

struct Data {
    channel: String,
    user: String,
    event: AckEvent,
}

#[derive(Debug)]
struct Task {
    event: AckEvent,
}

lazy_static! {
    static ref Q: Queue<Data> = Queue::new(10_000);
}

pub async fn queue(channel: String, user: String, event: AckEvent) {
    Q.push(Data {
        channel,
        user,
        event,
    })
    .await;
}

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
