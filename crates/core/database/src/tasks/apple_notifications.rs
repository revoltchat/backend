use std::io::Cursor;

use base64::{
    engine::{self},
    Engine as _,
};
use deadqueue::limited::Queue;
use once_cell::sync::Lazy;
use revolt_a2::{
    request::{
        notification::{DefaultAlert, NotificationOptions},
        payload::{APSAlert, APSSound, PayloadLike, APS},
    },
    Client, ClientConfig, Endpoint, Error, ErrorBody, ErrorReason, Priority, PushType, Response,
};
use revolt_config::config;
use revolt_models::v0::{Message, PushNotification};

use crate::Database;

/// Payload information, before assembly
#[derive(Debug)]
pub struct ApnPayload {
    message: Message,
    url: String,
    authorAvatar: String,
    authorDisplayName: String,
    channelName: String,
}

#[derive(Serialize, Debug)]
struct Payload<'a> {
    aps: APS<'a>,
    #[serde(skip_serializing)]
    options: NotificationOptions<'a>,
    #[serde(skip_serializing)]
    device_token: &'a str,

    message: &'a Message,
    url: &'a str,
    authorAvatar: &'a str,
    authorDisplayName: &'a str,
    channelName: &'a str,
}

impl<'a> PayloadLike for Payload<'a> {
    fn get_device_token(&self) -> &'a str {
        self.device_token
    }
    fn get_options(&self) -> &NotificationOptions {
        &self.options
    }
}

/// Task information
#[derive(Debug)]
pub struct AlertJob {
    /// Session Id
    session_id: String,

    /// Device token
    device_token: String,

    /// User Id
    user_id: String,

    /// Title
    title: String,

    /// Body
    body: String,

    /// Thread Id
    thread_id: String,

    /// Category (informs the client what kind of notification is being sent.)
    category: String,

    /// Payload used by the iOS client to modify the notification
    custom_payload: ApnPayload,
}

impl AlertJob {
    fn format_title(notification: &PushNotification) -> String {
        // ideally this changes depending on context
        // in a server, it would look like "Sendername, #channelname in servername"
        // in a group, it would look like "Sendername in groupname"
        // in a dm it should just be "Sendername".
        // not sure how feasible all those are given the PushNotification object as it currently stands.
        format!(
            "{} in {}",
            notification.author, notification.message.channel
        ) // TODO: this absolutely needs a channel name
    }
}

#[derive(Debug)]
pub struct BadgeJob {
    /// Session Id
    session_id: String,

    /// Device token
    device_token: String,

    /// User Id
    user_id: String,
}

#[derive(Debug)]
pub enum JobType {
    Alert(AlertJob),
    Badge(BadgeJob),
}

#[derive(Debug)]
pub struct ApnJob {
    job_type: JobType,
}

impl ApnJob {
    pub fn from_notification(
        session_id: String,
        user_id: String,
        device_token: String,
        notification: &PushNotification,
    ) -> ApnJob {
        ApnJob {
            job_type: JobType::Alert(AlertJob {
                session_id,
                device_token,
                user_id,
                title: AlertJob::format_title(notification),
                body: notification.body.to_string(),
                thread_id: notification.tag.to_string(),
                category: "ALERT_MESSAGE".to_string(),
                custom_payload: ApnPayload {
                    message: notification.message.clone(),
                    url: notification.url.clone(),
                    authorAvatar: notification.icon.clone(),
                    authorDisplayName: notification.author.clone(),
                    channelName: "#fetchchannelnamehere".to_string(), // TODO: get actual channel name
                },
            }),
        }
    }

    pub fn from_ack(session_id: String, user_id: String, device_token: String) -> ApnJob {
        ApnJob {
            job_type: JobType::Badge(BadgeJob {
                session_id,
                device_token,
                user_id,
            }),
        }
    }
}

enum AssembledPayload<'a> {
    Alert(Payload<'a>),
    Default(revolt_a2::request::payload::Payload<'a>),
}

static Q: Lazy<Queue<ApnJob>> = Lazy::new(|| Queue::new(10_000));

/// Queue a new task for a worker
pub async fn queue(task: ApnJob) {
    Q.try_push(task).ok();
    info!("Queue is using {} slots from {}.", Q.len(), Q.capacity());
}

async fn get_badge_count(db: &Database, user: &str) -> Option<u32> {
    if let Ok(unreads) = db.fetch_unreads(user).await {
        let mut mention_count = 0;
        for channel in unreads {
            if let Some(mentions) = channel.mentions {
                mention_count += mentions.len() as u32
            }
        }

        return Some(mention_count);
    }
    None
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

    let client_config = ClientConfig::new(endpoint);

    let client = Client::token(
        &mut Cursor::new(pkcs8),
        config.api.apn.key_id,
        config.api.apn.team_id,
        client_config,
    )
    .expect("could not create APN client");

    let payload_options = NotificationOptions {
        apns_id: None,
        apns_push_type: Some(PushType::Alert),
        apns_expiration: None,
        apns_priority: Some(Priority::High),
        apns_topic: Some("chat.revolt.app"),
        apns_collapse_id: None,
    };

    loop {
        let task = Q.pop().await;
        let payload: AssembledPayload;

        match task.job_type {
            JobType::Alert(ref alert) => {
                payload = AssembledPayload::Alert(Payload {
                    aps: APS {
                        alert: Some(APSAlert::Default(DefaultAlert {
                            title: Some(&alert.title),
                            subtitle: None,
                            body: Some(&alert.body),
                            title_loc_key: None,
                            title_loc_args: None,
                            action_loc_key: None,
                            loc_key: None,
                            loc_args: None,
                            launch_image: None,
                        })),
                        badge: get_badge_count(&db, &alert.user_id).await,
                        sound: Some(APSSound::Sound("default")),
                        thread_id: Some(&alert.thread_id),
                        content_available: None,
                        category: Some(&alert.category),
                        mutable_content: Some(1),
                        url_args: None,
                    },
                    device_token: &alert.device_token,
                    options: payload_options.clone(),
                    message: &alert.custom_payload.message,
                    url: &alert.custom_payload.url,
                    authorAvatar: &alert.custom_payload.authorAvatar,
                    authorDisplayName: &alert.custom_payload.authorDisplayName,
                    channelName: &alert.custom_payload.channelName,
                });
            }
            JobType::Badge(ref alert) => {
                payload = AssembledPayload::Default(revolt_a2::request::payload::Payload {
                    aps: APS {
                        alert: None,
                        badge: get_badge_count(&db, &alert.user_id).await,
                        sound: None,
                        thread_id: None,
                        content_available: None,
                        category: None,
                        mutable_content: None,
                        url_args: None,
                    },
                    device_token: &alert.device_token,
                    options: payload_options.clone(),
                    data: std::collections::BTreeMap::new(),
                })
            }
        }

        let resp = match payload {
            AssembledPayload::Alert(p) => client.send(p).await,
            AssembledPayload::Default(p) => client.send(p).await,
        };
        //println!("response from APNS: {:?}", resp);

        if let Err(err) = resp {
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
                        .remove_push_subscription_by_session_id(match task.job_type {
                            JobType::Alert(ref a) => &a.session_id.as_str(),
                            JobType::Badge(ref a) => &a.session_id.as_str(),
                        })
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
