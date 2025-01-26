use std::collections::HashMap;

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
                let name = alert
                    .from_user
                    .display_name
                    .or(Some(format!(
                        "{}#{}",
                        alert.from_user.username, alert.from_user.discriminator
                    )))
                    .clone()
                    .unwrap();

                let mut body = HashMap::new();
                body.insert("body", format!("{} sent you a friend request", name));

                payload_body = serde_json::to_string(&body).unwrap();
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

                let mut body = HashMap::new();
                body.insert("body", format!("{} accepted your friend request", name));

                payload_body = serde_json::to_string(&body).unwrap();
            }
            PayloadKind::Generic(alert) => {
                payload_body = serde_json::to_string(&alert).unwrap();
            }
            PayloadKind::MessageNotification(alert) => {
                payload_body = serde_json::to_string(&alert).unwrap();
            }
            PayloadKind::BadgeUpdate(_) => {
                panic!("Vapid cannot handle badge updates, and they should not be sent here.")
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
