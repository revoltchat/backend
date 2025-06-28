use async_channel::Receiver;
use async_std::{net::TcpStream, sync::Mutex};
use async_tungstenite::WebSocketStream;
use authifier::AuthifierEvent;
use futures::{pin_mut, select, stream::SplitSink, FutureExt, SinkExt};
use revolt_broker::event_stream;
use revolt_database::{events::client::EventV1, Database};
use sentry::Level;

use crate::{
    config::ProtocolConfiguration,
    events::state::{State, SubscriptionStateChange},
};

/// Event subscriber loop
pub async fn client_subscriber(
    write: &Mutex<SplitSink<WebSocketStream<TcpStream>, async_tungstenite::tungstenite::Message>>,
    cancelled: Receiver<()>,
    reloaded: Receiver<()>,
    protocol_config: &ProtocolConfiguration,
    db: &'static Database,
    state: &mut State,
) {
    let mut consumer = event_stream::Consumer::new().await;
    consumer.set_topics(state.subscribed.read().await.clone());

    let mut cancel = false;

    loop {
        // Reload consumer if subscriptions change
        if !matches!(state.apply_state().await, SubscriptionStateChange::None) {
            consumer.set_topics(state.subscribed.read().await.clone());
        }

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
                event = delivery => {
                    if let Some(mut event) = event {
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

                        break;
                    } else {
                        cancel = true;
                        break;
                    }
                }
            }
        }

        // Break out if cancelled
        if cancel {
            break;
        }
    }

    consumer.dispose_channel().await;
}
