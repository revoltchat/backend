use std::collections::HashMap;

use amqprs::{channel::Channel as AmqpChannel, consumer::AsyncConsumer, BasicProperties, Deliver};

use async_trait::async_trait;
use fcm::{Client, FcmError, FcmResponse, MessageBuilder};
use revolt_database::{events::rabbit::*, Database};
use revolt_models::v0::{Channel, PushNotification};

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

        if config.pushd.fcm.api_key.is_empty() {
            return Err("No FCM key present");
        }

        Ok(FcmOutboundConsumer {
            db,
            client: Client::new(),
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
        let resp: Result<FcmResponse, FcmError>;

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
                data.insert("type", "push.fr.receive");
                data.insert("id", &alert.from_user.id);
                data.insert("username", &name);

                let mut message_builder =
                    MessageBuilder::new(&config.pushd.fcm.api_key, &payload.token);
                _ = message_builder.data(&data);

                resp = self.client.send(message_builder.finalize()).await;
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

                let mut data = HashMap::new();
                data.insert("type", "push.fr.accepted");
                data.insert("id", &alert.accepted_user.id);
                data.insert("username", &name);

                let mut message_builder =
                    MessageBuilder::new(&config.pushd.fcm.api_key, &payload.token);
                _ = message_builder.data(&data);

                resp = self.client.send(message_builder.finalize()).await;
            }
            PayloadKind::Generic(alert) => {
                let mut notification = fcm::NotificationBuilder::new();
                notification.title(alert.title.as_str());
                if alert.icon.is_some() {
                    notification.icon(alert.icon.as_ref().unwrap());
                }
                notification.body(&alert.body);
                let notification = notification.finalize();

                let mut message_builder =
                    MessageBuilder::new(&config.pushd.fcm.api_key, &payload.token);
                message_builder.notification(notification);

                resp = self.client.send(message_builder.finalize()).await;
            }

            PayloadKind::MessageNotification(alert) => {
                let title = self.format_title(&alert);

                let mut notification = fcm::NotificationBuilder::new();
                notification.title(title.as_str());
                notification.icon(&alert.icon);
                notification.body(&alert.body);
                notification.tag(alert.channel.id());
                // TODO: expand support for fields
                let notification = notification.finalize();

                let mut message_builder =
                    MessageBuilder::new(&config.pushd.fcm.api_key, &payload.token);
                message_builder.notification(notification);

                resp = self.client.send(message_builder.finalize()).await;
            }
        }

        if let Err(err) = resp {
            match err {
                FcmError::Unauthorized => {
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
