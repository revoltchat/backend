// Queue Type: Linear
use async_channel::{ Sender, Receiver, bounded };
use mongodb::bson::{doc};
use web_push::{ContentEncoding, SubscriptionInfo, SubscriptionKeys, VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder};
use rauth::entities::{Model, Session};
use futures::StreamExt;

use crate::util::variables::VAPID_PRIVATE_KEY;
use crate::database::*;

struct Message {
    recipients: Vec<String>,
    payload: String
}

lazy_static! {
    static ref CHANNEL: (Sender<Message>, Receiver<Message>) = bounded(100);
}

pub async fn queue(recipients: Vec<String>, payload: String) {
    CHANNEL.0.send(Message { recipients, payload }).await.ok();
}

pub async fn run() {
    let client = WebPushClient::new();
    let key = base64::decode_config(VAPID_PRIVATE_KEY.clone(), base64::URL_SAFE)
        .expect("valid `VAPID_PRIVATE_KEY`");

    while let Ok(Message { recipients, payload }) = CHANNEL.1.recv().await {
        if let Ok(mut cursor) = Session::find(
            &get_db(),
            doc! {
                "_id": {
                    "$in": recipients
                },
                "subscription": {
                    "$exists": true
                }
            },
            None
        )
        .await {
            while let Some(Ok(session)) = cursor.next().await {
                if let Some(sub) = session.subscription {
                    let subscription = SubscriptionInfo {
                        endpoint: sub.endpoint,
                        keys: SubscriptionKeys {
                            auth: sub.auth,
                            p256dh: sub.p256dh
                        }
                    };

                    let mut builder = WebPushMessageBuilder::new(&subscription).unwrap();
                    let sig_builder = VapidSignatureBuilder::from_pem(
                        std::io::Cursor::new(&key),
                        &subscription,
                    )
                    .unwrap();

                    let signature = sig_builder.build().unwrap();
                    builder.set_vapid_signature(signature);
                    builder.set_payload(ContentEncoding::AesGcm, payload.as_bytes());

                    let msg = builder.build().unwrap();
                    client.send(msg).await.ok();
                }
            }
        }
    }
}
