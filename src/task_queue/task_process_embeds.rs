// Queue Type: Linear
use async_channel::{ Sender, Receiver, bounded };
use log::info;
use mongodb::bson::{doc, to_bson};

use crate::{database::*, notifications::events::ClientboundNotification};

struct Message {
    channel: String,
    id: String,
    content: String
}

lazy_static! {
    static ref CHANNEL: (Sender<Message>, Receiver<Message>) = bounded(100);
}

pub async fn queue(channel: String, id: String, content: String) {
    CHANNEL.0.send(Message { channel, id, content }).await.ok();
}

pub async fn run() {
    let messages = get_collection("messages");

    while let Ok(Message { channel, id, content }) = CHANNEL.1.recv().await {
        if let Ok(embeds) = Embed::generate(content).await {
            if let Ok(bson) = to_bson(&embeds) {
                if let Ok(_) = messages
                    .update_one(
                        doc! {
                            "_id": &id
                        },
                        doc! {
                            "$push": {
                                "embeds": {
                                    "$each": bson
                                }
                            }
                        },
                        None,
                    )
                    .await
                {
                    info!("Generated embeds for {}.", &id);
                    ClientboundNotification::MessageUpdate {
                        id,
                        channel: channel.clone(),
                        data: json!({ "embeds": embeds }),
                    }
                    .publish(channel);
                }
            }
        }
    }
}
