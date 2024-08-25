use amqprs::{channel::Channel as AmqpChannel, consumer::AsyncConsumer, BasicProperties, Deliver};

use async_trait::async_trait;
use base64::{
    engine::{self},
    Engine as _,
};
use revolt_database::{events::rabbit::*, Database};
// use revolt_models::v0::{Channel, PushNotification};
use web_push::{
    ContentEncoding, IsahcWebPushClient, SubscriptionInfo, SubscriptionKeys, VapidSignatureBuilder,
    WebPushClient, WebPushError, WebPushMessageBuilder,
};

pub struct VapidOutboundConsumer {
    db: Database,
    client: IsahcWebPushClient,
    pkey: Vec<u8>,
}

// impl VapidOutboundConsumer {
//     fn format_title(&self, notification: &PushNotification) -> String {
//         // ideally this changes depending on context
//         // in a server, it would look like "Sendername, #channelname in servername"
//         // in a group, it would look like "Sendername in groupname"
//         // in a dm it should just be "Sendername".
//         // not sure how feasible all those are given the PushNotification object as it currently stands.

//         match &notification.channel {
//             Channel::DirectMessage { .. } => notification.author.clone(),
//             Channel::Group { name, .. } => format!("{}, #{}", notification.author, name),
//             Channel::TextChannel { name, .. } | Channel::VoiceChannel { name, .. } => {
//                 format!("{} in #{}", notification.author, name)
//             }
//             _ => "Unknown".to_string(),
//         }
//     }
// }

impl VapidOutboundConsumer {
    pub async fn new(db: Database) -> Result<VapidOutboundConsumer, &'static str> {
        let config = revolt_config::config().await;

        if config.pushd.vapid.private_key.is_empty() | config.pushd.vapid.public_key.is_empty() {
            return Err("No Vapid keys present");
        }

        let web_push_private_key = engine::general_purpose::URL_SAFE_NO_PAD
            .decode(config.pushd.vapid.private_key)
            .expect("valid `VAPID_PRIVATE_KEY`");

        Ok(VapidOutboundConsumer {
            db,
            client: IsahcWebPushClient::new().unwrap(),
            pkey: web_push_private_key,
        })
    }
}

#[allow(unused_variables)]
#[async_trait]
impl AsyncConsumer for VapidOutboundConsumer {
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

        let subscription = SubscriptionInfo {
            endpoint: payload.extras.get("endpoint").unwrap().clone(),
            keys: SubscriptionKeys {
                auth: payload.token,
                p256dh: payload.extras.get("p256dh").unwrap().clone(),
            },
        };

        #[allow(clippy::needless_late_init)]
        let payload_body: String;

        match payload.notification {
            PayloadKind::FRReceived(alert) => {
                payload_body = serde_json::to_string(&alert).unwrap();
            }
            PayloadKind::FRAccepted(alert) => {
                payload_body = serde_json::to_string(&alert).unwrap();
            }
            PayloadKind::Generic(alert) => {
                payload_body = serde_json::to_string(&alert).unwrap();
            }
            PayloadKind::MessageNotification(alert) => {
                payload_body = serde_json::to_string(&alert).unwrap();
            }
        }

        match VapidSignatureBuilder::from_pem(std::io::Cursor::new(&self.pkey), &subscription) {
            Ok(sig_builder) => match sig_builder.build() {
                Ok(signature) => {
                    let mut builder = WebPushMessageBuilder::new(&subscription);
                    builder.set_vapid_signature(signature);

                    builder.set_payload(ContentEncoding::AesGcm, payload_body.as_bytes());

                    match builder.build() {
                        Ok(msg) => {
                            if let Err(err) = self.client.send(msg).await {
                                if err == WebPushError::Unauthorized {
                                    if let Err(err) = self
                                        .db
                                        .remove_push_subscription_by_session_id(&payload.session_id)
                                        .await
                                    {
                                        revolt_config::capture_error(&err);
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            revolt_config::capture_error(&err);
                        }
                    }
                }
                Err(err) => {
                    revolt_config::capture_error(&err);
                }
            },
            Err(err) => {
                revolt_config::capture_error(&err);
            }
        }
    }
}
