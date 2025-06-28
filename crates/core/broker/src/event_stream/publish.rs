use lapin::{
    Error,
    protocol::basic::AMQPProperties,
    publisher_confirm::PublisherConfirm,
    types::{AMQPValue, FieldTable},
};
use revolt_config::config;
use serde::Serialize;

use crate::event_stream::{create_channel, get_connection};

/// Publish an event to the message broker
pub async fn publish_event<T: Serialize>(
    channel: &str,
    data: &T,
) -> Result<PublisherConfirm, Error> {
    let config = config().await;

    let mut headers: FieldTable = Default::default();
    headers.insert(
        "x-stream-filter-value".into(),
        AMQPValue::LongString(channel.into()),
    );

    let conn = get_connection().await;

    create_channel(&conn, config.rabbit.event_stream.clone())
        .await
        .basic_publish(
            &config.rabbit.event_stream.exchange,
            &config.rabbit.event_stream.queue,
            Default::default(),
            &rmp_serde::to_vec_named(data).unwrap(),
            AMQPProperties::default().with_headers(headers),
        )
        .await
}
