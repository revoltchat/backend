use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_std::stream::StreamExt;
use lapin::{
    Channel,
    options::BasicAckOptions,
    types::{AMQPValue, FieldArray, FieldTable, LongLongInt},
};
use rand::Rng;
use revolt_config::config;
use serde::de::DeserializeOwned;

use crate::event_stream::get_channel;

pub struct Consumer {
    channel: Arc<Channel>,
    tag: String,
    topics: HashSet<String>,
    topics_changed: bool,
    consumer: Option<lapin::Consumer>,
    offset: Option<LongLongInt>,
}

impl Consumer {
    /// Create a new event stream consumer
    pub async fn new() -> Consumer {
        Consumer {
            channel: get_channel().await,
            tag: rand::rng()
                .sample_iter::<char, _>(&rand::distr::StandardUniform)
                .take(32)
                .collect(),
            topics: HashSet::new(),
            topics_changed: false,
            consumer: None,
            offset: None,
        }
    }

    /// Update the set of topics
    pub fn set_topics(&mut self, topics: HashSet<String>) {
        self.topics = topics;
        self.topics_changed = true;
    }

    /// Get the current consumer
    pub async fn ensure_consumer(&mut self) {
        if self.topics_changed {
            self.consumer = None;
        }

        if self.consumer.is_none() {
            let config = config().await;

            // Build arguments for consumer
            let mut args: FieldTable = Default::default();

            // Configure stream filter to select topics we are listening for
            {
                let mut filter: FieldArray = Default::default();
                for topic in &self.topics {
                    filter.push(AMQPValue::LongString(topic.as_str().into()));
                }

                args.insert("x-stream-filter".into(), AMQPValue::FieldArray(filter));
            }

            // Set stream offset if applicable
            if let Some(offset) = self.offset {
                args.insert("x-stream-offset".into(), AMQPValue::LongLongInt(offset));
            }

            // Create the consumer
            self.consumer = Some(
                self.channel
                    .basic_consume(
                        &config.rabbit.event_stream.queue,
                        &self.tag,
                        Default::default(),
                        args,
                    )
                    .await
                    .unwrap(),
            );
        }
    }

    /// Close the active consumer
    pub async fn dispose(&mut self) {
        // Close the channel -- don't do this actually
        // capture_internal_error!(self.channel.close(0, "closing channel").await);

        // Close the consumer
        if let Err(err) = self
            .channel
            .basic_cancel(&self.tag, Default::default())
            .await
        {
            eprintln!("Failed to close consumer! {:?}", err);
        }
        // is this necessary?
        // else {
        // Read the consumer to the end
        //     while let Some(delivery) = consumer.next().await {
        //         let delivery = delivery.expect("error in consumer");
        //         delivery.ack(BasicAckOptions::default()).await.expect("ack");
        //     }
        // }
    }

    /// Get the next item
    pub async fn next<T: DeserializeOwned>(&mut self) -> Option<T> {
        self.ensure_consumer().await;

        let consumer = self.consumer.as_mut().unwrap();

        while let Some(Ok(delivery)) = consumer.next().await {
            // Acknowledgement is required
            delivery.ack(BasicAckOptions::default()).await.expect("ack");

            // Parse the delivery headers
            let headers: HashMap<String, AMQPValue> = delivery
                .properties
                .headers()
                .as_ref()
                .map(|table| {
                    table
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v.clone()))
                        .collect()
                })
                .unwrap_or_default();

            // Keep track of the current offset
            let stream_offset = headers
                .get("x-stream-offset")
                .expect("`x-stream-offset` not present in message!");

            self.offset = Some(stream_offset.as_long_long_int().unwrap() + 1);

            // Client-side topic filtering (broker uses Bloom filter so may have false-positives)
            let filter_value = headers
                .get("x-stream-filter-value")
                .expect("`x-stream-filter-value` not present in message!")
                .as_long_string()
                .expect("`string`")
                .to_string();

            if self.topics.contains(&filter_value) {
                // Deserialise the data
                return Some(rmp_serde::from_slice(&delivery.data).expect("`data`"));
            }
        }

        None
    }
}
