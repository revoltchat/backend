use amqprs::{
    channel::{BasicPublishArguments, Channel},
    consumer::AsyncConsumer,
    BasicProperties, Deliver,
};
use async_trait::async_trait;
use revolt_database::{events::rabbit::*, Database};

pub struct OriginConsumer {
    db: Database,
    authifier_db: authifier::Database,
    config: revolt_config::Settings,
}

impl OriginConsumer {
    pub fn new(
        db: Database,
        authifier_db: authifier::Database,
        config: revolt_config::Settings,
    ) -> OriginConsumer {
        OriginConsumer {
            db,
            authifier_db,
            config,
        }
    }
}

#[allow(unused_variables)]
#[async_trait]
impl AsyncConsumer for OriginConsumer {
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
            for session in sessions {
                if let Some(sub) = session.subscription {
                    let sendable = serde_json::to_string(&PayloadToService {
                        notification: PayloadKind::MessageNotification(
                            payload.notification.clone(),
                        ),
                        token: sub.auth,
                    })
                    .unwrap();

                    let args: BasicPublishArguments;

                    if sub.endpoint == "apn" {
                        args = BasicPublishArguments::new(
                            self.config.pushd.exchange.as_str(),
                            self.config.pushd.apn.queue.as_str(),
                        )
                        .finish();
                    } else if sub.endpoint == "fcm" {
                        args = BasicPublishArguments::new(
                            self.config.pushd.exchange.as_str(),
                            self.config.pushd.fcm.queue.as_str(),
                        )
                        .finish();
                    } else {
                        // web push (vapid)
                        args = BasicPublishArguments::new(
                            self.config.pushd.exchange.as_str(),
                            self.config.pushd.vapid.queue.as_str(),
                        )
                        .finish();
                    }

                    channel
                        .basic_publish(BasicProperties::default(), sendable.into(), args)
                        .await
                        .unwrap();
                }
            }
        }
    }
}
