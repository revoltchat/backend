use crate::bson::doc;
use crate::util::variables::delta::VAPID_PRIVATE_KEY;

use deadqueue::limited::Queue;
use rauth::Database;
use web_push::{
    ContentEncoding, SubscriptionInfo, SubscriptionKeys, VapidSignatureBuilder, WebPushClient,
    WebPushMessageBuilder,
};

/// Task information
#[derive(Debug)]
struct PushTask {
    /// User IDs of the targets that are to receive this notification
    recipients: Vec<String>,
    /// Raw JSON payload to send to clients
    payload: String,
}

lazy_static! {
    static ref Q: Queue<PushTask> = Queue::new(10_000);
}

/// Queue a new task for a worker
pub async fn queue(recipients: Vec<String>, payload: String) {
    if recipients.is_empty() {
        return;
    }

    Q.try_push(PushTask {
        recipients,
        payload,
    })
    .ok();

    info!("Queue is using {} slots from {}.", Q.len(), Q.capacity());
}

/// Start a new worker
pub async fn worker(db: Database) {
    let client = WebPushClient::new();
    let key = base64::decode_config(VAPID_PRIVATE_KEY.clone(), base64::URL_SAFE)
        .expect("valid `VAPID_PRIVATE_KEY`");

    loop {
        let task = Q.pop().await;

        if let Ok(sessions) = db.find_sessions_with_subscription(&task.recipients).await {
            for session in sessions {
                if let Some(sub) = session.subscription {
                    let subscription = SubscriptionInfo {
                        endpoint: sub.endpoint,
                        keys: SubscriptionKeys {
                            auth: sub.auth,
                            p256dh: sub.p256dh,
                        },
                    };

                    match WebPushMessageBuilder::new(&subscription) {
                        Ok(mut builder) => {
                            match VapidSignatureBuilder::from_pem(
                                std::io::Cursor::new(&key),
                                &subscription,
                            ) {
                                Ok(sig_builder) => match sig_builder.build() {
                                    Ok(signature) => {
                                        builder.set_vapid_signature(signature);
                                        builder.set_payload(
                                            ContentEncoding::AesGcm,
                                            task.payload.as_bytes(),
                                        );

                                        match builder.build() {
                                            Ok(msg) => match client.send(msg).await {
                                                Ok(_) => {
                                                    info!(
                                                        "Sent Web Push notification to {:?}.",
                                                        session.id
                                                    )
                                                }
                                                Err(err) => {
                                                    error!("Hit error sending Web Push! {:?}", err)
                                                }
                                            },
                                            Err(err) => {
                                                error!(
                                                    "Failed to build message for {}! {:?}",
                                                    session.user_id, err
                                                )
                                            }
                                        }
                                    }
                                    Err(err) => error!(
                                        "Failed to build signature for {}! {:?}",
                                        session.user_id, err
                                    ),
                                },
                                Err(err) => error!(
                                    "Failed to create signature builder for {}! {:?}",
                                    session.user_id, err
                                ),
                            }
                        }
                        Err(err) => error!(
                            "Invalid subscription information for {}! {:?}",
                            session.user_id, err
                        ),
                    }
                }
            }
        }
    }
}
