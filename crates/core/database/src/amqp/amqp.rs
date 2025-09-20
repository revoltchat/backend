use std::collections::HashSet;

use crate::events::rabbit::*;
use crate::User;
use amqprs::channel::{BasicPublishArguments, ExchangeDeclareArguments};
use amqprs::connection::OpenConnectionArguments;
use amqprs::{channel::Channel, connection::Connection, error::Error as AMQPError};
use amqprs::{BasicProperties, FieldTable};
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

    pub async fn new_auto() -> AMQP {
        let config = revolt_config::config().await;

        let connection = Connection::open(&OpenConnectionArguments::new(
            &config.rabbit.host,
            config.rabbit.port,
            &config.rabbit.username,
            &config.rabbit.password,
        ))
        .await
        .expect("Failed to connect to RabbitMQ");

        let channel = connection
            .open_channel(None)
            .await
            .expect("Failed to open RabbitMQ channel");

        channel
            .exchange_declare(
                ExchangeDeclareArguments::new(&config.pushd.exchange, "direct")
                    .durable(true)
                    .finish(),
            )
            .await
            .expect("Failed to declare exchange");

        AMQP::new(connection, channel)
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

        debug!(
            "Sending friend request accept payload on channel {}: {}",
            config.pushd.get_fr_accepted_routing_key(),
            payload
        );
        self.channel
            .basic_publish(
                BasicProperties::default()
                    .with_content_type("application/json")
                    .with_persistence(true)
                    .finish(),
                payload.into(),
                BasicPublishArguments::new(
                    &config.pushd.exchange,
                    &config.pushd.get_fr_accepted_routing_key(),
                ),
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

        debug!(
            "Sending friend request received payload on channel {}: {}",
            config.pushd.get_fr_received_routing_key(),
            payload
        );

        self.channel
            .basic_publish(
                BasicProperties::default()
                    .with_content_type("application/json")
                    .with_persistence(true)
                    .finish(),
                payload.into(),
                BasicPublishArguments::new(
                    &config.pushd.exchange,
                    &config.pushd.get_fr_received_routing_key(),
                ),
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

        debug!(
            "Sending generic payload on channel {}: {}",
            config.pushd.get_generic_routing_key(),
            payload
        );

        self.channel
            .basic_publish(
                BasicProperties::default()
                    .with_content_type("application/json")
                    .with_persistence(true)
                    .finish(),
                payload.into(),
                BasicPublishArguments::new(
                    &config.pushd.exchange,
                    &config.pushd.get_generic_routing_key(),
                ),
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

        debug!(
            "Sending message payload on channel {}: {}",
            config.pushd.get_message_routing_key(),
            payload
        );

        self.channel
            .basic_publish(
                BasicProperties::default()
                    .with_content_type("application/json")
                    .with_persistence(true)
                    .finish(),
                payload.into(),
                BasicPublishArguments::new(
                    &config.pushd.exchange,
                    &config.pushd.get_message_routing_key(),
                ),
            )
            .await
    }

    pub async fn mass_mention_message_sent(
        &self,
        server_id: String,
        payload: Vec<PushNotification>,
    ) -> Result<(), AMQPError> {
        let config = revolt_config::config().await;

        let payload = MassMessageSentPayload {
            notifications: payload,
            server_id,
        };
        let payload = to_string(&payload).unwrap();

        let routing_key = config.pushd.get_mass_mention_routing_key();

        debug!(
            "Sending mass mention payload on channel {}: {}",
            routing_key, payload
        );

        self.channel
            .basic_publish(
                BasicProperties::default()
                    .with_content_type("application/json")
                    .with_persistence(true)
                    .finish(),
                payload.into(),
                BasicPublishArguments::new(&config.pushd.exchange, routing_key.as_str()),
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
            user_id: user_id.clone(),
            channel_id: channel_id.clone(),
            message_id,
        };
        let payload = to_string(&payload).unwrap();

        info!(
            "Sending ack payload on channel {}: {}",
            config.pushd.ack_queue, payload
        );

        let mut headers = FieldTable::new();
        headers.insert(
            "x-deduplication-header".try_into().unwrap(),
            format!("{}-{}", &user_id, &channel_id).into(),
        );

        self.channel
            .basic_publish(
                BasicProperties::default()
                    .with_content_type("application/json")
                    .with_persistence(true)
                    //.with_headers(headers)
                    .finish(),
                payload.into(),
                BasicPublishArguments::new(&config.pushd.exchange, &config.pushd.ack_queue),
            )
            .await
    }

    /// # DM Call Update
    /// Used to send an update about a DM call, eg. start or end of a call.
    /// Recipients can be used to narrow the scope of recipients, otherwise all recipients will be notified.
    /// `ended` refers to the ringing period, not necessarily the call itself.
    pub async fn dm_call_updated(
        &self,
        initiator_id: &str,
        channel_id: &str,
        started_at: Option<&str>,
        ended: bool,
        recipients: Option<Vec<String>>,
    ) -> Result<(), AMQPError> {
        let config = revolt_config::config().await;

        let payload = InternalDmCallPayload {
            payload: DmCallPayload {
                initiator_id: initiator_id.to_string(),
                channel_id: channel_id.to_string(),
                started_at: started_at.map(|f| f.to_string()),
                ended,
            },
            recipients,
        };
        let payload = to_string(&payload).unwrap();

        debug!(
            "Sending dm call update payload on channel {}: {}",
            config.pushd.get_dm_call_routing_key(),
            payload
        );

        self.channel
            .basic_publish(
                BasicProperties::default()
                    .with_content_type("application/json")
                    .with_persistence(true)
                    .finish(),
                payload.into(),
                BasicPublishArguments::new(
                    &config.pushd.exchange,
                    &config.pushd.get_dm_call_routing_key(),
                ),
            )
            .await
    }
}
