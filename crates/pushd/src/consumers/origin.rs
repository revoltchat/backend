use std::collections::HashMap;

use amqprs::{
    channel::{BasicPublishArguments, Channel},
    consumer::AsyncConsumer,
    BasicProperties, Deliver,
};
use async_trait::async_trait;
use revolt_database::{events::rabbit::*, Database};

pub struct OriginMessageConsumer {
    #[allow(dead_code)]
    db: Database,
    authifier_db: authifier::Database,
}

impl OriginMessageConsumer {
    pub fn new(db: Database, authifier_db: authifier::Database) -> OriginMessageConsumer {
        OriginMessageConsumer { db, authifier_db }
    }
}

#[allow(unused_variables)]
#[async_trait]
impl AsyncConsumer for OriginMessageConsumer {
    /// This consumer handles delegating messages into their respective platform queues.
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let content = String::from_utf8(content).unwrap();
        let payload: MessageSentNotification = serde_json::from_str(content.as_str()).unwrap();

        if let Ok(sessions) = self
            .authifier_db
            .find_sessions_with_subscription(&payload.users)
            .await
        {
            let config = revolt_config::config().await;
            for session in sessions {
                if let Some(sub) = session.subscription {
                    let mut sendable = PayloadToService {
                        notification: PayloadKind::MessageNotification(
                            payload.notification.clone(),
                        ),
                        token: sub.auth,
                        user_id: session.user_id,
                        session_id: session.id,
                        extras: HashMap::new(),
                    };

                    let args: BasicPublishArguments;

                    if sub.endpoint == "apn" {
                        args = BasicPublishArguments::new(
                            config.pushd.exchange.as_str(),
                            config.pushd.apn.queue.as_str(),
                        )
                        .finish();
                    } else if sub.endpoint == "fcm" {
                        args = BasicPublishArguments::new(
                            config.pushd.exchange.as_str(),
                            config.pushd.fcm.queue.as_str(),
                        )
                        .finish();
                    } else {
                        // web push (vapid)
                        args = BasicPublishArguments::new(
                            config.pushd.exchange.as_str(),
                            config.pushd.vapid.queue.as_str(),
                        )
                        .finish();
                        sendable.extras.insert("p265dh".to_string(), sub.p256dh);
                        sendable.extras.insert("endpoint".to_string(), sub.endpoint);
                    }

                    let payload = serde_json::to_string(&sendable).unwrap();

                    channel
                        .basic_publish(BasicProperties::default(), payload.into(), args)
                        .await
                        .unwrap();
                }
            }
        }
    }
}
