use std::collections::HashMap;

use async_channel::Receiver;
use async_std::{net::TcpStream, sync::Mutex};
use async_tungstenite::WebSocketStream;
use authifier::AuthifierEvent;
use futures::{pin_mut, select, stream::SplitSink, FutureExt, SinkExt, StreamExt};
use lapin::{
    options::BasicAckOptions,
    types::{AMQPValue, FieldArray, FieldTable, LongLongInt},
};
use rand::Rng;
use revolt_config::report_internal_error;
use revolt_database::{
    events::client::{get_event_stream_channel, EventV1},
    Database,
};
use sentry::Level;

use crate::{
    config::ProtocolConfiguration,
    events::state::{State, SubscriptionStateChange},
};

static QUEUE_NAME: &str = "revolt.events";

/// Event subscriber loop
pub async fn client_subscriber(
    write: &Mutex<SplitSink<WebSocketStream<TcpStream>, async_tungstenite::tungstenite::Message>>,
    cancelled: Receiver<()>,
    reloaded: Receiver<()>,
    protocol_config: &ProtocolConfiguration,
    db: &'static Database,
    state: &mut State,
) {
    // * --- CHANNEL CREATE ---
    let channel = get_event_stream_channel().await;
    // * --- CHANNEL END ---

    let mut offset: Option<LongLongInt> = None;
    let mut cancel = false;

    loop {
        // Build arguments for consumer
        let mut args: FieldTable = Default::default();

        // Configure stream filter to select topics we are listening for
        {
            let mut filter: FieldArray = Default::default();
            for topic in state.subscribed.read().await.iter() {
                filter.push(AMQPValue::LongString(topic.as_str().into()));
            }

            args.insert("x-stream-filter".into(), AMQPValue::FieldArray(filter));
        }

        // Set stream offset if applicable
        if let Some(offset) = offset {
            args.insert("x-stream-offset".into(), AMQPValue::LongLongInt(offset));
        }

        // Create the consumer
        let tag: String = rand::thread_rng()
            .sample_iter::<char, _>(&rand::distributions::Standard)
            .take(32)
            .collect();

        let mut consumer = channel
            .basic_consume(QUEUE_NAME, &tag, Default::default(), args)
            .await
            .unwrap();

        // Read incoming events
        loop {
            let reloaded = reloaded.recv().fuse();
            let cancelled = cancelled.recv().fuse();
            let delivery = consumer.next().fuse();
            pin_mut!(delivery, reloaded, cancelled);

            select! {
                _ = reloaded => {
                    break;
                }
                _ = cancelled => {
                    cancel = true;
                    break;
                }
                delivery = delivery => {
                    let delivery = delivery.expect("error in consumer").expect("error in consumer");

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

                    // Client-side topic filtering (broker uses Bloom filter so may have false-positives)
                    let filter_value = headers
                        .get("x-stream-filter-value")
                        .expect("`x-stream-filter-value` not present in message!");

                    if state
                        .subscribed
                        .read()
                        .await
                        .contains(
                            &filter_value
                            .as_long_string()
                            .expect("`string`")
                            .to_string()
                        ) {
                        // Deserialise the data
                        let mut event: EventV1 = rmp_serde::from_slice(&delivery.data).expect("`data`");

                        // Handle the event
                        if let EventV1::Auth(auth) = &event {
                            if let AuthifierEvent::DeleteSession { session_id, .. } = auth {
                                if &state.session_id == session_id {
                                    event = EventV1::Logout;
                                }
                            } else if let AuthifierEvent::DeleteAllSessions {
                                exclude_session_id, ..
                            } = auth
                            {
                                if let Some(excluded) = exclude_session_id {
                                    if &state.session_id != excluded {
                                        event = EventV1::Logout;
                                    }
                                } else {
                                    event = EventV1::Logout;
                                }
                            }
                        } else {
                            let should_send = state.handle_incoming_event_v1(db, &mut event).await;
                            if !should_send {
                                continue;
                            }
                        }

                        let result = write.lock().await.send(protocol_config.encode(&event)).await;
                        if let Err(e) = result {
                            use async_tungstenite::tungstenite::Error;
                            if !matches!(e, Error::AlreadyClosed | Error::ConnectionClosed) {
                                let err = format!("Error while sending an event: {e:?}");
                                warn!("{}", err);
                                sentry::capture_message(&err, Level::Warning);
                            }

                            cancel = true;
                            break;
                        }

                        if let EventV1::Logout = event {
                            info!("User {} received log out event!", state.user_id);
                            cancel = true;
                            break;
                        }
                    }

                    // Keep track of the current offset
                    let stream_offset = headers
                        .get("x-stream-offset")
                        .expect("`x-stream-offset` not present in message!");

                    offset = Some(stream_offset.as_long_long_int().unwrap() + 1);

                    // Reload consumer if subscriptions change
                    if !matches!(state.apply_state().await, SubscriptionStateChange::None) {
                        break;
                    }
                }
            }
        }

        // Close the consumer
        if let Err(err) = channel.basic_cancel(&tag, Default::default()).await {
            eprintln!("Failed to close consumer! {:?}", err);
        } else {
            // Read the consumer to the end
            while let Some(delivery) = consumer.next().await {
                let delivery = delivery.expect("error in consumer");
                delivery.ack(BasicAckOptions::default()).await.expect("ack");
            }
        }

        // Break out if cancelled
        if cancel {
            break;
        }
    }

    report_internal_error!(channel.close(0, "closing channel").await).ok();
}
