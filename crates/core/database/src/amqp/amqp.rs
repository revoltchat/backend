use std::collections::HashSet;

use crate::events::rabbit::*;
use crate::User;
use amqprs::channel::BasicPublishArguments;
use amqprs::BasicProperties;
use amqprs::{channel::Channel, connection::Connection, error::Error as AMQPError};
use revolt_models::v0::PushNotification;
use revolt_presence::filter_online;

use serde_json::to_string;

#[derive(Clone)]
pub struct AMQP {
    #[allow(unused)]
    connection: Connection,
    channel: Channel,
}

impl AMQP {
    pub fn new(connection: Connection, channel: Channel) -> AMQP {
        AMQP {
            connection,
            channel,
        }
    }

    pub async fn friend_request_accepted(
        &self,
        accepted_request_user: &User,
        sent_request_user: &User,
    ) -> Result<(), AMQPError> {
        let config = revolt_config::config().await;
        let payload = FRAcceptedPayload {
            accepted_user: accepted_request_user.to_owned(),
            user: sent_request_user.id.clone(),
        };
        let payload = to_string(&payload).unwrap();

        info!("Sending friend request accepted event: {}", &payload);

        self.channel
            .basic_publish(
                BasicProperties::default()
                    .with_content_type("application/json")
                    .with_persistence(true)
                    .finish(),
                payload.into(),
                BasicPublishArguments::new(&config.pushd.exchange, &config.pushd.fr_accepted_queue),
            )
            .await
    }

    pub async fn friend_request_received(
        &self,
        received_request_user: &User,
        sent_request_user: &User,
    ) -> Result<(), AMQPError> {
        let config = revolt_config::config().await;
        let payload = FRReceivedPayload {
            from_user: sent_request_user.to_owned(),
            user: received_request_user.id.clone(),
        };
        let payload = to_string(&payload).unwrap();

        info!("Sending friend request received event: {}", &payload);

        self.channel
            .basic_publish(
                BasicProperties::default()
                    .with_content_type("application/json")
                    .with_persistence(true)
                    .finish(),
                payload.into(),
                BasicPublishArguments::new(&config.pushd.exchange, &config.pushd.fr_received_queue),
            )
            .await
    }

    pub async fn generic_message(
        &self,
        user: &User,
        title: String,
        body: String,
        icon: Option<String>,
    ) -> Result<(), AMQPError> {
        let config = revolt_config::config().await;
        let payload = GenericPayload {
            title,
            body,
            icon,
            user: user.to_owned(),
        };
        let payload = to_string(&payload).unwrap();

        self.channel
            .basic_publish(
                BasicProperties::default()
                    .with_content_type("application/json")
                    .with_persistence(true)
                    .finish(),
                payload.into(),
                BasicPublishArguments::new(&config.pushd.exchange, &config.pushd.generic_queue),
            )
            .await
    }

    pub async fn message_sent(
        &self,
        recipients: Vec<String>,
        payload: PushNotification,
    ) -> Result<(), AMQPError> {
        if recipients.is_empty() {
            return Ok(());
        }

        let config = revolt_config::config().await;

        let online_ids = filter_online(&recipients).await;
        let recipients = (&recipients.into_iter().collect::<HashSet<String>>() - &online_ids)
            .into_iter()
            .collect::<Vec<String>>();

        let payload = MessageSentPayload {
            notification: payload,
            users: recipients,
        };
        let payload = to_string(&payload).unwrap();

        self.channel
            .basic_publish(
                BasicProperties::default()
                    .with_content_type("application/json")
                    .with_persistence(true)
                    .finish(),
                payload.into(),
                BasicPublishArguments::new(&config.pushd.exchange, &config.pushd.message_queue),
            )
            .await
    }

    pub async fn ack_message(
        &self,
        user_id: String,
        channel_id: String,
        message_id: String,
    ) -> Result<(), AMQPError> {
        let config = revolt_config::config().await;

        let payload = AckPayload {
            user_id,
            channel_id,
            message_id,
        };
        let payload = to_string(&payload).unwrap();

        self.channel
            .basic_publish(
                BasicProperties::default()
                    .with_content_type("application/json")
                    .with_persistence(true)
                    .finish(),
                payload.into(),
                BasicPublishArguments::new(&config.pushd.exchange, &config.pushd.message_queue),
            )
            .await
    }
}
