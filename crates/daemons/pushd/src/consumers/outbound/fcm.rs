use std::{collections::HashMap, time::Duration};

use amqprs::{channel::Channel as AmqpChannel, consumer::AsyncConsumer, BasicProperties, Deliver};

use async_trait::async_trait;
use fcm_v1::{
    android::AndroidConfig,
    auth::{Authenticator, ServiceAccountKey},
    message::{Message, Notification},
    Client, Error as FcmError,
};
use revolt_database::{events::rabbit::*, Database};
use revolt_models::v0::{Channel, PushNotification};
use serde_json::Value;

pub struct FcmOutboundConsumer {
    db: Database,
    client: Client,
}

impl FcmOutboundConsumer {
    fn format_title(&self, notification: &PushNotification) -> String {
        // ideally this changes depending on context
        // in a server, it would look like "Sendername, #channelname in servername"
        // in a group, it would look like "Sendername in groupname"
        // in a dm it should just be "Sendername".
        // not sure how feasible all those are given the PushNotification object as it currently stands.

        match &notification.channel {
            Channel::DirectMessage { .. } => notification.author.clone(),
            Channel::Group { name, .. } => format!("{}, #{}", notification.author, name),
            Channel::TextChannel { name, .. } | Channel::VoiceChannel { name, .. } => {
                format!("{} in #{}", notification.author, name)
            }
            _ => "Unknown".to_string(),
        }
    }
}

impl FcmOutboundConsumer {
    pub async fn new(db: Database) -> Result<FcmOutboundConsumer, &'static str> {
        let config = revolt_config::config().await;

        Ok(FcmOutboundConsumer {
            db,
            client: Client::new(
                Authenticator::service_account::<&str>(ServiceAccountKey {
                    key_type: Some(config.pushd.fcm.key_type),
                    project_id: Some(config.pushd.fcm.project_id.clone()),
                    private_key_id: Some(config.pushd.fcm.private_key_id),
                    private_key: config.pushd.fcm.private_key,
                    client_email: config.pushd.fcm.client_email,
                    client_id: Some(config.pushd.fcm.client_id),
                    auth_uri: Some(config.pushd.fcm.auth_uri),
                    token_uri: config.pushd.fcm.token_uri,
                    auth_provider_x509_cert_url: Some(config.pushd.fcm.auth_provider_x509_cert_url),
                    client_x509_cert_url: Some(config.pushd.fcm.client_x509_cert_url),
                })
                .await
                .unwrap(),
                config.pushd.fcm.project_id,
                false,
                Duration::from_secs(5),
            ),
        })
    }
}

#[allow(unused_variables)]
#[async_trait]
impl AsyncConsumer for FcmOutboundConsumer {
    async fn consume(
        &mut self,
        channel: &AmqpChannel,
        deliver: Deliver,
        basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let content = String::from_utf8(content).unwrap();
        let payload: PayloadToService = serde_json::from_str(content.as_str()).unwrap();

        let config = revolt_config::config().await;

        #[allow(clippy::needless_late_init)]
        let resp: Result<Message, FcmError>;

        match payload.notification {
            PayloadKind::FRReceived(alert) => {
                let name = alert
                    .from_user
                    .display_name
                    .or(Some(format!(
                        "{}#{}",
                        alert.from_user.username, alert.from_user.discriminator
                    )))
                    .clone()
                    .unwrap();

                let mut data = HashMap::new();
                data.insert(
                    "type".to_string(),
                    Value::String("push.fr.receive".to_string()),
                );
                data.insert("id".to_string(), Value::String(alert.from_user.id));
                data.insert("username".to_string(), Value::String(name));

                let msg = Message {
                    token: Some(payload.token),
                    data: Some(data),
                    ..Default::default()
                };

                resp = self.client.send(&msg).await;
            }

            PayloadKind::FRAccepted(alert) => {
                let name = alert
                    .accepted_user
                    .display_name
                    .or(Some(format!(
                        "{}#{}",
                        alert.accepted_user.username, alert.accepted_user.discriminator
                    )))
                    .clone()
                    .unwrap();

                let mut data: HashMap<String, Value> = HashMap::new();
                data.insert(
                    "type".to_string(),
                    Value::String("push.fr.accept".to_string()),
                );
                data.insert("id".to_string(), Value::String(alert.accepted_user.id));
                data.insert("username".to_string(), Value::String(name));

                let msg = Message {
                    token: Some(payload.token),
                    data: Some(data),
                    ..Default::default()
                };

                resp = self.client.send(&msg).await;
            }
            PayloadKind::Generic(alert) => {
                let msg = Message {
                    token: Some(payload.token),
                    notification: Some(Notification {
                        title: Some(alert.title),
                        body: Some(alert.body),
                        image: alert.icon,
                    }),
                    ..Default::default()
                };

                resp = self.client.send(&msg).await;
            }

            PayloadKind::MessageNotification(alert) => {
                let title = self.format_title(&alert);

                let msg = Message {
                    token: Some(payload.token),
                    notification: Some(Notification {
                        title: Some(title),
                        body: Some(alert.body),
                        image: Some(alert.icon),
                    }),
                    android: Some(AndroidConfig {
                        collapse_key: Some(alert.tag),
                        ..Default::default()
                    }),
                    ..Default::default()
                };

                resp = self.client.send(&msg).await;
            }

            PayloadKind::BadgeUpdate(_) => {
                panic!("FCM cannot handle badge updates, and they should not be sent here.")
            }
        }

        if let Err(err) = resp {
            match err {
                FcmError::Auth => {
                    if let Err(err) = self
                        .db
                        .remove_push_subscription_by_session_id(&payload.session_id)
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
