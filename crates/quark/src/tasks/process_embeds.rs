use crate::util::variables::delta::{JANUARY_URL, MAX_EMBED_COUNT, JANUARY_CONCURRENT_CONNECTIONS};
use crate::{
    models::{message::AppendMessage, Message},
    types::january::Embed,
    Database,
};

use async_lock::Semaphore;
use async_std::task::spawn;
use deadqueue::limited::Queue;
use std::sync::Arc;
use once_cell::sync::Lazy;

/// Task information
#[derive(Debug)]
struct EmbedTask {
    /// Channel we're processing the event in
    channel: String,
    /// ID of the message we're processing
    id: String,
    /// Content of the message
    content: String,
}

static Q: Lazy<Queue<EmbedTask>> = Lazy::new(|| Queue::new(10_000));


/// Queue a new task for a worker
pub async fn queue(channel: String, id: String, content: String) {
    Q.try_push(EmbedTask {
        channel,
        id,
        content,
    })
    .ok();

    info!("Queue is using {} slots from {}.", Q.len(), Q.capacity());
}

/// Start a new worker
pub async fn worker(db: Database) {
    let semaphore = Arc::new(Semaphore::new(*JANUARY_CONCURRENT_CONNECTIONS));

    loop {
        let task = Q.pop().await;
        let db = db.clone();
        let semaphore = semaphore.clone();

        spawn(async move {
            let embeds = Embed::generate(task.content, &JANUARY_URL, *MAX_EMBED_COUNT, semaphore).await;

            if let Ok(embeds) = embeds {
                if let Err(err) = Message::append(
                    &db,
                    task.id,
                    task.channel,
                    AppendMessage {
                        embeds: Some(embeds),
                    },
                )
                .await
                {
                    error!("Encountered an error appending to message: {:?}", err);
                }
            }
        });
    }
}
