use crate::util::variables::VAPID_PRIVATE_KEY;

use deadqueue::limited::Queue;
use futures::StreamExt;
use rauth::entities::Session;
use revolt_quark::{bson::doc, r#impl::mongo::MongoDb, Database};
use web_push::{
    ContentEncoding, SubscriptionInfo, SubscriptionKeys, VapidSignatureBuilder, WebPushClient,
    WebPushMessageBuilder,
};

#[derive(Debug)]
struct PushTask {
    recipients: Vec<String>,
    payload: String,
}

lazy_static! {
    static ref Q: Queue<PushTask> = Queue::new(10_000);
}

pub async fn queue(recipients: Vec<String>, payload: String) {
    if recipients.is_empty() {
        return;
    }

    Q.push(PushTask {
        recipients,
        payload,
    })
    .await;
}

pub async fn worker(db: Database) {
    let client = WebPushClient::new();
    let key = base64::decode_config(VAPID_PRIVATE_KEY.clone(), base64::URL_SAFE)
        .expect("valid `VAPID_PRIVATE_KEY`");

    if let Database::MongoDb(MongoDb(db)) = db {
        loop {
            let task = Q.pop().await;

            // ! FIXME: this is hard-coded until rauth is merged into quark
            if let Ok(mut cursor) = db
                .database("revolt")
                .collection::<Session>("sessions")
                .find(
                    doc! {
                        "user_id": {
                            "$in": task.recipients
                        },
                        "subscription": {
                            "$exists": true
                        }
                    },
                    None,
                )
                .await
            {
                while let Some(Ok(session)) = cursor.next().await {
                    if let Some(sub) = session.subscription {
                        let subscription = SubscriptionInfo {
                            endpoint: sub.endpoint,
                            keys: SubscriptionKeys {
                                auth: sub.auth,
                                p256dh: sub.p256dh,
                            },
                        };

                        let mut builder = WebPushMessageBuilder::new(&subscription).unwrap();
                        let sig_builder = VapidSignatureBuilder::from_pem(
                            std::io::Cursor::new(&key),
                            &subscription,
                        )
                        .unwrap();

                        let signature = sig_builder.build().unwrap();
                        builder.set_vapid_signature(signature);
                        builder.set_payload(ContentEncoding::AesGcm, task.payload.as_bytes());

                        let msg = builder.build().unwrap();
                        match client.send(msg).await {
                            Ok(_) => info!("Sent Web Push notification to {:?}.", session.id),
                            Err(err) => error!("Hit error sending Web Push! {:?}", err),
                        }
                    }
                }
            }
        }
    }
}
