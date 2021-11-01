// Queue Type: Linear
use async_channel::{ Sender, Receiver, bounded };
use mongodb::bson::{doc, to_bson};

use crate::{database::*, notifications::events::ClientboundNotification};

type Message = (String, String, String);

lazy_static! {
    static ref CHANNEL: (Sender<Message>, Receiver<Message>) = bounded(100);
}

pub async fn queue(channel: String, id: String, content: String) {
    CHANNEL.0.send((channel, id, content)).await.ok();
}

pub async fn run() {
    let messages = get_collection("messages");

    while let Ok((channel, id, content)) = CHANNEL.1.recv().await {
        if let Ok(embeds) = Embed::generate(content).await {
            if let Ok(bson) = to_bson(&embeds) {
                if let Ok(_) = messages
                    .update_one(
                        doc! {
                            "_id": &id
                        },
                        doc! {
                            "$set": {
                                "embeds": bson
                            }
                        },
                        None,
                    )
                    .await
                {
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
