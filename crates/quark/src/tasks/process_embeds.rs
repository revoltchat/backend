use crate::util::variables::delta::{JANUARY_URL, MAX_EMBED_COUNT};
use crate::{
    models::{message::AppendMessage, Message},
    types::january::Embed,
    Database,
};

use deadqueue::limited::Queue;

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

lazy_static! {
    static ref Q: Queue<EmbedTask> = Queue::new(10_000);
}

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
    loop {
        let task = Q.pop().await;
        if let Ok(embeds) = Embed::generate(task.content, &*JANUARY_URL, *MAX_EMBED_COUNT).await {
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
    }
}
