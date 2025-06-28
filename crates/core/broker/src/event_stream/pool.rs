use async_std::sync::Mutex;
use lapin::{
    Channel,
    options::QueueDeclareOptions,
    types::{AMQPValue, FieldTable},
};
use log::{debug, warn};
use revolt_config::{RabbitEventStream, config};
use std::sync::Arc;

use crate::create_client;

/// Get a handle to the event stream
pub async fn get_channel() -> Arc<Channel> {
    let config = config().await;

    static CONNECTIONS: Mutex<Vec<Arc<lapin::Channel>>> = Mutex::new(Vec::new());

    let mut connections = CONNECTIONS.lock().await;
    connections.retain(|item| {
        if item.status().connected() {
            true
        } else {
            warn!(
                "Dropping connection with status {:?}",
                item.status().state()
            );

            false
        }
    });

    debug!(
        "Connections: {}, Clients: {:?}",
        connections.len(),
        connections
            .iter()
            .map(Arc::strong_count)
            .collect::<Vec<usize>>()
    );

    for channel in connections.iter() {
        if Arc::strong_count(channel) < config.rabbit.event_stream.channels_per_conn {
            return channel.clone();
        }
    }

    let conn = create_client().await;
    let channel = Arc::new(create_channel(conn, config.rabbit.event_stream).await);

    connections.push(channel.clone());
    channel
}

/// Create a channel
async fn create_channel(
    conn: lapin::Connection,
    event_stream: RabbitEventStream,
) -> lapin::Channel {
    let channel = conn.create_channel().await.unwrap();

    let mut args: FieldTable = Default::default();

    args.insert(
        // set queue type to stream
        "x-queue-type".into(),
        AMQPValue::LongString("stream".into()),
    );

    args.insert(
        // max. size of the stream
        "x-max-length-bytes".into(),
        AMQPValue::LongLongInt(event_stream.stream_max_length_bytes),
    );

    args.insert(
        // size of the Bloom filter
        "x-stream-filter-size-bytes".into(),
        AMQPValue::LongLongInt(event_stream.filter_size_bytes),
    );

    channel
        .queue_declare(
            &event_stream.queue,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            args,
        )
        .await
        .unwrap();

    channel
        .basic_qos(event_stream.qos_prefetch, Default::default())
        .await
        .unwrap();

    channel
}
