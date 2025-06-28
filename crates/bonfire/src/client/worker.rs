use std::{collections::HashSet, sync::Arc};

use async_channel::{Receiver, Sender};
use async_std::{
    net::TcpStream,
    sync::{Mutex, RwLock},
};
use async_tungstenite::WebSocketStream;
use futures::{
    pin_mut, select,
    stream::{SplitSink, SplitStream},
    FutureExt, SinkExt, TryStreamExt,
};
use revolt_database::events::{client::EventV1, server::ClientMessage};
use sentry::Level;

use crate::{config::ProtocolConfiguration, events::state::State};

pub struct WorkerRef {
    user_id: String,
    active_servers: Arc<Mutex<lru_time_cache::LruCache<String, ()>>>,
    subscribed: Arc<RwLock<HashSet<String>>>,
}

impl WorkerRef {
    pub fn from(state: &State) -> WorkerRef {
        WorkerRef {
            user_id: state.user_id.clone(),
            active_servers: state.active_servers.clone(),
            subscribed: state.subscribed.clone(),
        }
    }
}

/// Incoming message handling
pub async fn client_worker(
    mut read: SplitStream<WebSocketStream<TcpStream>>,
    write: &Mutex<SplitSink<WebSocketStream<TcpStream>, async_tungstenite::tungstenite::Message>>,
    cancelled: Receiver<()>,
    reload: Sender<()>,
    config: &ProtocolConfiguration,
    state: WorkerRef,
) {
    loop {
        let read = read.try_next().fuse();
        let cancelled = cancelled.recv().fuse();
        pin_mut!(read, cancelled);

        select! {
            _ = cancelled => { return; },
            msg = read => {
                let msg = match msg {
                    Ok(Some(msg)) => msg,
                    Ok(None) => {
                        warn!("Received a None message!");
                        sentry::capture_message("Received a None message!", Level::Warning);
                        return;
                    }
                    Err(e) => {
                        use async_tungstenite::tungstenite::Error;
                        if !matches!(e, Error::AlreadyClosed | Error::ConnectionClosed) {
                            let err = format!("Error while reading an event: {e:?}");
                            warn!("{}", err);
                            sentry::capture_message(&err, Level::Warning);
                        }

                        return;
                    }
                };

                let Ok(payload) = config.decode(&msg) else {
                    continue;
                };

                match payload {
                    ClientMessage::BeginTyping { channel } => {
                        if !state.subscribed.read().await.contains(&channel) {
                            continue;
                        }

                        EventV1::ChannelStartTyping {
                            id: channel.clone(),
                            user: state.user_id.clone(),
                        }
                        .p(channel.clone())
                        .await;
                    }
                    ClientMessage::EndTyping { channel } => {
                        if !state.subscribed.read().await.contains(&channel) {
                            continue;
                        }

                        EventV1::ChannelStopTyping {
                            id: channel.clone(),
                            user: state.user_id.clone(),
                        }
                        .p(channel.clone())
                        .await;
                    }
                    ClientMessage::Subscribe { server_id } => {
                        let mut servers = state.active_servers.lock().await;
                        let has_item = servers.contains_key(&server_id);
                        servers.insert(server_id, ());

                        if !has_item {
                            // Poke the listener to adjust subscriptions
                            reload.send(()).await.ok();
                        }
                    }
                    ClientMessage::Ping { data, responded } => {
                        if responded.is_none() {
                            write
                                .lock()
                                .await
                                .send(config.encode(&EventV1::Pong { data }))
                                .await
                                .ok();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
