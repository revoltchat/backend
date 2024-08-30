use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use authifier::Database;
use base64::{
    engine::{self},
    Engine as _,
};
use deadqueue::limited::Queue;
use fcm_v1::auth::{Authenticator, ServiceAccountKey};
use once_cell::sync::Lazy;
use revolt_config::config;
use revolt_models::v0::PushNotification;
use revolt_presence::filter_online;
use serde_json::json;
use web_push::{
    ContentEncoding, IsahcWebPushClient, SubscriptionInfo, SubscriptionKeys, VapidSignatureBuilder,
    WebPushClient, WebPushMessageBuilder,
};

use super::apple_notifications;

/// Task information
#[derive(Debug)]
struct PushTask {
    /// User IDs of the targets that are to receive this notification
    recipients: Vec<String>,
    /// Push Notification
    payload: PushNotification,
}

static Q: Lazy<Queue<PushTask>> = Lazy::new(|| Queue::new(10_000));

/// Queue a new task for a worker
pub async fn queue(recipients: Vec<String>, payload: PushNotification) {
    if recipients.is_empty() {
        return;
    }

    let online_ids = filter_online(&recipients).await;
    let recipients = (&recipients.into_iter().collect::<HashSet<String>>() - &online_ids)
        .into_iter()
        .collect::<Vec<String>>();

    Q.try_push(PushTask {
        recipients,
        payload,
    })
    .ok();

    info!("Queue is using {} slots from {}.", Q.len(), Q.capacity());
}

/// Start a new worker
pub async fn worker(db: Database) {
    let config = config().await;

    let web_push_client = IsahcWebPushClient::new().unwrap();
    let fcm_client = if config.api.fcm.key_type.is_empty() {
        None
    } else {
        Some(fcm_v1::Client::new(
            Authenticator::service_account::<&str>(ServiceAccountKey {
                key_type: Some(config.api.fcm.key_type),
                project_id: Some(config.api.fcm.project_id.clone()),
                private_key_id: Some(config.api.fcm.private_key_id),
                private_key: config.api.fcm.private_key,
                client_email: config.api.fcm.client_email,
                client_id: Some(config.api.fcm.client_id),
                auth_uri: Some(config.api.fcm.auth_uri),
                token_uri: config.api.fcm.token_uri,
                auth_provider_x509_cert_url: Some(config.api.fcm.auth_provider_x509_cert_url),
                client_x509_cert_url: Some(config.api.fcm.client_x509_cert_url),
            })
            .await
            .unwrap(),
            config.api.fcm.project_id,
            false,
            Duration::from_secs(5),
        ))
    };

    let web_push_private_key = engine::general_purpose::URL_SAFE_NO_PAD
        .decode(config.api.vapid.private_key)
        .expect("valid `VAPID_PRIVATE_KEY`");

    loop {
        let task = Q.pop().await;

        if let Ok(sessions) = db.find_sessions_with_subscription(&task.recipients).await {
            for session in sessions {
                if let Some(sub) = session.subscription {
                    if sub.endpoint == "fcm" {
                        // Use Firebase Cloud Messaging
                        if let Some(client) = &fcm_client {
                            let PushNotification {
                                author,
                                icon,
                                image,
                                body,
                                tag: _,
                                timestamp: _,
                                url: _,
                                message: _,
                            } = &task.payload;

                            let message = fcm_v1::message::Message {
                                token: Some(sub.auth),
                                data: Some(HashMap::from([
                                    (
                                        "author".to_owned(),
                                        serde_json::Value::String(author.clone()),
                                    ),
                                    ("icon".to_owned(), serde_json::Value::String(icon.clone())),
                                    (
                                        "image".to_owned(),
                                        if let Some(image) = image {
                                            serde_json::Value::String(image.clone())
                                        } else {
                                            serde_json::Value::Null
                                        },
                                    ),
                                    ("body".to_owned(), serde_json::Value::String(body.clone())),
                                ])),
                                ..Default::default()
                            };

                            if let Err(err) = client.send(&message).await {
                                error!("Failed to send FCM notification! {:?}", err);
                            } else {
                                info!("Sent FCM notification to {:?}.", session.id);
                            }
                        } else {
                            info!("No FCM token was specified!");
                        }
                    } else if sub.endpoint == "apn" {
                        apple_notifications::queue(apple_notifications::ApnJob::from_notification(
                            session.id,
                            session.user_id,
                            sub.auth,
                            &task.payload,
                        ))
                        .await;
                    } else {
                        // Use Web Push Standard
                        let subscription = SubscriptionInfo {
                            endpoint: sub.endpoint,
                            keys: SubscriptionKeys {
                                auth: sub.auth,
                                p256dh: sub.p256dh,
                            },
                        };

                        match VapidSignatureBuilder::from_pem(
                            std::io::Cursor::new(&web_push_private_key),
                            &subscription,
                        ) {
                            Ok(sig_builder) => match sig_builder.build() {
                                Ok(signature) => {
                                    let mut builder = WebPushMessageBuilder::new(&subscription);
                                    builder.set_vapid_signature(signature);

                                    let payload = json!(task.payload).to_string();
                                    builder
                                        .set_payload(ContentEncoding::AesGcm, payload.as_bytes());

                                    match builder.build() {
                                        Ok(msg) => match web_push_client.send(msg).await {
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
                }
            }
        }
    }
}
