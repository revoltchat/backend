use std::io::Cursor;

use base64::{
    engine::{self},
    Engine as _,
};
use deadqueue::limited::Queue;
use once_cell::sync::Lazy;
use revolt_a2::{Client, ClientConfig, DefaultNotificationBuilder, Endpoint};
use revolt_a2::{Error, ErrorBody, ErrorReason, NotificationBuilder, Response};
use revolt_config::config;
use revolt_models::v0::{Message, PushNotification};

use crate::Database;

/// Payload information
#[derive(Debug)]
pub struct ApnCustomPayload {
    message: Message,
    serverId: String,
    authorAvatar: String,
    authorDisplayName: String,
    channelName: String,
}

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

    /// Category (informs the client what kind of notification is being sent.)
    category: String,

    /// Payload used by the iOS client to modify the notification
    payload: ApnCustomPayload,
}

impl ApnTask {
    fn format_title(notification: &PushNotification) -> String {
        // ideally this changes depending on context
        // in a server, it would look like "Sendername, #channelname in servername"
        // in a group, it would look like "Sendername in groupname"
        // in a dm it should just be "Sendername".
        // not sure how feasible all those are given the PushNotification object as it currently stands.
        todo!();
    }

    pub fn from_notification(
        session_id: String,
        device_token: String,
        notification: &PushNotification,
    ) -> ApnTask {
        ApnTask {
            session_id,
            device_token,
            title: ApnTask::format_title(notification),
            body: notification.body.to_string(),
            thread_id: notification.tag.to_string(),
            category: "ALERT_MESSAGE".to_string(),
            payload: ApnCustomPayload {
                message: (),
                serverId: (),
                authorAvatar: notification.icon.to_string(),
                authorDisplayName: notification.author.to_string(),
                channelName: (),
            },
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

    let endpoint = if config.api.apn.sandbox {
        Endpoint::Sandbox
    } else {
        Endpoint::Production
    };

    let pkcs8 = engine::general_purpose::STANDARD
        .decode(config.api.apn.pkcs8)
        .expect("valid `pcks8`");

    let client_config = ClientConfig::default();
    client_config.endpoint = endpoint;

    let client = Client::token(
        &mut Cursor::new(pkcs8),
        config.api.apn.key_id,
        config.api.apn.team_id,
        client_config,
    )
    .expect("could not create APN client");

    loop {
        let task = Q.pop().await;
        let payload = DefaultNotificationBuilder::new()
            .set_title(&task.title)
            .set_body(&task.body)
            .set_thread_id(&task.thread_id)
            .set_category(&task.category)
            .set_mutable_content() // allows the service extension to execute
            .build(&task.device_token, Default::default());

        payload.data = &task.payload; // this looks like a job for someone more rust-inclined than me. -tom

        println!("sending APNS payload: {:?}", payload.data);

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
