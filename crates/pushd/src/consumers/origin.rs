use std::collections::HashMap;

use amqprs::{
    channel::{BasicPublishArguments, Channel},
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
    BasicProperties, Deliver,
};
use async_trait::async_trait;
use revolt_database::{events::rabbit::*, Database};
use tracing::debug;

pub struct OriginMessageConsumer {
    #[allow(dead_code)]
    db: Database,
    authifier_db: authifier::Database,
    conn: Option<Connection>,
    channel: Option<Channel>,
}

impl OriginMessageConsumer {
    pub fn new(db: Database, authifier_db: authifier::Database) -> OriginMessageConsumer {
        OriginMessageConsumer {
            db,
            authifier_db,
            conn: None,
            channel: None,
        }
    }

    async fn make_channel(&mut self) {
        let config = revolt_config::config().await;

        let args = OpenConnectionArguments::new(
            &config.rabbit.host,
            config.rabbit.port,
            &config.rabbit.username,
            &config.rabbit.password,
        );
        self.conn = Some(amqprs::connection::Connection::open(&args).await.unwrap());

        let _raw_channel = self
            .conn
            .as_ref()
            .unwrap()
            .open_channel(None)
            .await
            .unwrap();

        self.channel = Some(_raw_channel);
    }

    async fn publish_message(
        &mut self,
        payload: Vec<u8>,
        args: BasicPublishArguments,
        attempt: u8,
    ) {
        let routing_key = &args.routing_key.clone();
        if attempt > 3 {
            panic!(
                "Failed 3 attempts to send a message to queue {}",
                routing_key
            );
        }
        if self.channel.is_none() {
            self.make_channel().await;
        }

        if let Some(chnl) = &self.channel {
            //if let Err(err) =
            chnl.basic_publish(BasicProperties::default(), payload.clone(), args.clone())
                .await
                .unwrap();
            // {
            //     match err {
            //         Error::InternalChannelError(_) => {
            //             self.make_channel().await;
            //             self.publish_message(payload, args, attempt + 1).await;
            //         }
            //         _ => {}
            //     }
            // }
            debug!("Sent message to queue for target {}", routing_key);
        }
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

        debug!("Received message event on origin");

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
                        sendable
                            .extras
                            .insert("endpoint".to_string(), sub.endpoint.clone());
                    }

                    let payload = serde_json::to_string(&sendable).unwrap();

                    self.publish_message(payload.into(), args, 1).await;
                }
            }
        }
    }
}
