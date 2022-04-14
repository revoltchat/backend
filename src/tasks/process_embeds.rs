use crate::util::variables::{JANUARY_URL, MAX_EMBED_COUNT};

use deadqueue::limited::Queue;
use log::error;
use revolt_quark::{
    models::{message::AppendMessage, Message},
    types::january::Embed,
    Database,
};

#[derive(Debug)]
struct EmbedTask {
    channel: String,
    id: String,
    content: String,
}

lazy_static! {
    static ref Q: Queue<EmbedTask> = Queue::new(10_000);
}

pub async fn queue(channel: String, id: String, content: String) {
    Q.push(EmbedTask {
        channel,
        id,
        content,
    })
    .await;
}

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
