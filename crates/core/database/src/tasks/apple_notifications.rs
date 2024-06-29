use std::io::Cursor;

use base64::{
    engine::{self},
    Engine as _,
};
use deadqueue::limited::Queue;
use once_cell::sync::Lazy;
use revolt_a2::{Client, ClientConfig, DefaultNotificationBuilder};
use revolt_a2::{Error, ErrorBody, ErrorReason, NotificationBuilder, Response};
use revolt_config::config;
use revolt_models::v0::PushNotification;

use crate::Database;

/// Task information
#[derive(Debug)]
pub struct ApnTask {
    /// Session Id
    session_id: String,

    /// Device token
    device_token: String,

    /// Title
    title: String,

    /// Body
    body: String,

    /// Thread Id
    thread_id: String,
}

impl ApnTask {
    pub fn from_notification(
        session_id: String,
        device_token: String,
        notification: &PushNotification,
    ) -> ApnTask {
        ApnTask {
            session_id,
            device_token,
            title: notification.author.to_string(),
            body: notification.body.to_string(),
            thread_id: notification.tag.to_string(),
        }
    }
}

static Q: Lazy<Queue<ApnTask>> = Lazy::new(|| Queue::new(10_000));

/// Queue a new task for a worker
pub async fn queue(task: ApnTask) {
    Q.try_push(task).ok();
    info!("Queue is using {} slots from {}.", Q.len(), Q.capacity());
}

/// Start a new worker
pub async fn worker(db: Database) {
    let config = config().await;
    if config.api.apn.pkcs8.is_empty()
        || config.api.apn.key_id.is_empty()
        || config.api.apn.team_id.is_empty()
    {
        eprintln!("Missing APN keys.");
        return;
    }

    let pkcs8 = engine::general_purpose::STANDARD
        .decode(config.api.apn.pkcs8)
        .expect("valid `pcks8`");

    let client = Client::token(
        &mut Cursor::new(pkcs8),
        config.api.apn.key_id,
        config.api.apn.team_id,
        ClientConfig::default(),
    )
    .expect("could not create APN client");

    loop {
        let task = Q.pop().await;
        let payload = DefaultNotificationBuilder::new()
            .set_title(&task.title)
            .set_body(&task.body)
            .set_thread_id(&task.thread_id)
            .build(&task.device_token, Default::default());

        if let Err(err) = client.send(payload).await {
            match err {
                Error::ResponseError(Response {
                    error:
                        Some(ErrorBody {
                            reason: ErrorReason::BadDeviceToken | ErrorReason::Unregistered,
                            ..
                        }),
                    ..
                }) => {
                    if let Err(err) = db
                        .remove_push_subscription_by_session_id(&task.session_id)
                        .await
                    {
                        revolt_config::capture_error(&err);
                    }
                }
                err => {
                    revolt_config::capture_error(&err);
                }
            }
        }
    }
}
