use std::{collections::HashSet, net::SocketAddr, sync::Arc};

use async_tungstenite::WebSocketStream;
use authifier::AuthifierEvent;
use fred::{
    interfaces::{ClientLike, EventInterface, PubsubInterface},
    types::RedisConfig,
};
use futures::{
    channel::oneshot,
    pin_mut, select,
    stream::{SplitSink, SplitStream},
    FutureExt, SinkExt, StreamExt, TryStreamExt,
};
use redis_kiss::{PayloadType, REDIS_PAYLOAD_TYPE, REDIS_URI};
use revolt_database::{
    events::{client::EventV1, server::ClientMessage},
    Database, User, UserHint,
};
use revolt_presence::{create_session, delete_session};

use async_std::{
    net::TcpStream,
    sync::{Mutex, RwLock},
};
use revolt_result::create_error;
use sentry::Level;

use crate::config::{ProtocolConfiguration, WebsocketHandshakeCallback};
use crate::events::state::{State, SubscriptionStateChange};

type WsReader = SplitStream<WebSocketStream<TcpStream>>;
type WsWriter = SplitSink<WebSocketStream<TcpStream>, async_tungstenite::tungstenite::Message>;

/// Start a new WebSocket client worker given access to the database,
/// the relevant TCP stream and the remote address of the client.
pub async fn client(db: &'static Database, stream: TcpStream, addr: SocketAddr) {
    // Upgrade the TCP connection to a WebSocket connection.
    // In this process, we also parse any additional parameters given.
    // e.g. wss://example.com?format=json&version=1
    let (sender, receiver) = oneshot::channel();
    let Ok(ws) = async_tungstenite::accept_hdr_async_with_config(
        stream,
        WebsocketHandshakeCallback::from(sender),
        None,
    )
    .await
    else {
        return;
    };

    // Verify we've received a valid config, otherwise we should just drop the connection.
    let Ok(mut config) = receiver.await else {
        return;
    };

    info!(
        "User {addr:?} provided protocol configuration (version = {}, format = {:?})",
        config.get_protocol_version(),
        config.get_protocol_format()
    );

    // Split the socket for simultaneously read and write.
    let (mut write, mut read) = ws.split();

    // If the user has not provided authentication, request information.
    if config.get_session_token().is_none() {
        while let Ok(Some(message)) = read.try_next().await {
            if let Ok(ClientMessage::Authenticate { token }) = config.decode(&message) {
                config.set_session_token(token);
                break;
            }
        }
    }

    // Try to authenticate the user.
    let Some(token) = config.get_session_token().as_ref() else {
        write
            .send(config.encode(&create_error!(InvalidSession)))
            .await
            .ok();
        return;
    };

    let (user, session_id) = match User::from_token(db, token, UserHint::Any).await {
        Ok(user) => user,
        Err(err) => {
            write.send(config.encode(&err)).await.ok();
            return;
        }
    };

    info!("User {addr:?} authenticated as @{}", user.username);

    // Create local state.
    let mut state = State::from(user, session_id);
    let user_id = state.cache.user_id.clone();

    // Notify socket we have authenticated.
    if let Err(err) = write.send(config.encode(&EventV1::Authenticated)).await {
        error!("Failed to write: {err:?}");
        sentry::capture_error(&err);
        return;
    }

    // Download required data to local cache and send Ready payload.
    let ready_payload = match state.generate_ready_payload(db).await {
        Ok(ready_payload) => ready_payload,
        Err(err) => {
            sentry::capture_error(&err);
            return;
        }
    };

    if let Err(err) = write.send(config.encode(&ready_payload)).await {
        error!("Failed to write: {err:?}");
        sentry::capture_error(&err);
        return;
    }

    // Create presence session.
    let (first_session, session_id) = create_session(&user_id, 0).await;

    // If this was the first session, notify other users that we just went online.
    if first_session {
        state.broadcast_presence_change(true).await;
    }

    {
        let write = Mutex::new(write);
        let subscribed = state.subscribed.clone();

        // Create a PubSub connection to poll on.
        let listener = listener(db, &mut state, addr, &config, &write).fuse();
        // Read from WebSocket stream.
        let worker = worker(addr, subscribed, user_id.clone(), &config, read, &write).fuse();

        // Pin both tasks.
        pin_mut!(listener, worker);

        // Wait for either disconnect or for listener to die.
        select!(
            () = listener => {},
            () = worker => {}
        );
    }
    // Clean up presence session.
    let last_session = delete_session(&user_id, session_id).await;

    // If this was the last session, notify other users that we just went offline.
    if last_session {
        state.broadcast_presence_change(false).await;
    }
}

async fn listener(
    db: &'static Database,
    state: &mut State,
    addr: SocketAddr,
    config: &ProtocolConfiguration,
    write: &Mutex<WsWriter>,
) {
    let redis_config = RedisConfig::from_url(&REDIS_URI).unwrap();

    let subscriber = match fred::types::Builder::from_config(redis_config).build_subscriber_client()
    {
        Ok(subscriber) => subscriber,
        Err(err) => {
            error!("Failed to build a subscriber: {err:?}");
            sentry::capture_error(&err);
            return;
        }
    };

    if let Err(err) = subscriber.init().await {
        error!("Failed to init subscriber: {err:?}");
        sentry::capture_error(&err);
        return;
    };

    let mut message_rx = subscriber.message_rx();
    loop {
        // Check for state changes for subscriptions.
        match state.apply_state().await {
            SubscriptionStateChange::Reset => {
                if let Err(err) = subscriber.unsubscribe_all().await {
                    error!("Unsubscribe all failed: {err:?}");
                    sentry::capture_error(&err);
                    return;
                }

                let subscribed = state.subscribed.read().await;
                for id in subscribed.iter() {
                    if let Err(err) = subscriber.subscribe(id).await {
                        error!("Subscribe failed: {err:?}");
                        sentry::capture_error(&err);
                        return;
                    }
                }

                #[cfg(debug_assertions)]
                info!("{addr:?} has reset their subscriptions");
            }
            SubscriptionStateChange::Change { add, remove } => {
                for id in remove {
                    #[cfg(debug_assertions)]
                    info!("{addr:?} unsubscribing from {id}");

                    if let Err(err) = subscriber.unsubscribe(id).await {
                        error!("Unsubscribe failed: {err:?}");
                        sentry::capture_error(&err);
                        return;
                    }
                }

                for id in add {
                    #[cfg(debug_assertions)]
                    info!("{addr:?} subscribing to {id}");

                    if let Err(err) = subscriber.subscribe(id).await {
                        error!("Subscribe failed: {err:?}");
                        sentry::capture_error(&err);
                        return;
                    }
                }
            }
            SubscriptionStateChange::None => {}
        }

        // Handle incoming events.
        let message = match message_rx.recv().await {
            Ok(message) => message,
            Err(e) => {
                error!("Error while consuming pub/sub messages: {e:?}");
                sentry::capture_error(&e);
                return;
            }
        };

        let event = match *REDIS_PAYLOAD_TYPE {
            PayloadType::Json => message
                .value
                .as_str()
                .and_then(|s| serde_json::from_str::<EventV1>(s.as_ref()).ok()),
            PayloadType::Msgpack => message
                .value
                .as_bytes()
                .and_then(|b| rmp_serde::from_slice::<EventV1>(b).ok()),
            PayloadType::Bincode => message
                .value
                .as_bytes()
                .and_then(|b| bincode::deserialize::<EventV1>(b).ok()),
        };

        let Some(mut event) = event else {
            let err = format!(
                "Failed to deserialise an event for {}! Introspection: `{:?}`",
                message.channel,
                message
                    .value
                    .as_string()
                    .map(|x| x.chars().take(32).collect::<String>())
            );

            error!("{}", err);
            sentry::capture_message(&err, Level::Error);
            return;
        };

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

        let result = write.lock().await.send(config.encode(&event)).await;
        if let Err(e) = result {
            use async_tungstenite::tungstenite::Error;
            if !matches!(e, Error::AlreadyClosed | Error::ConnectionClosed) {
                let err = format!("Error while sending an event to {addr:?}: {e:?}");
                warn!("{}", err);
                sentry::capture_message(&err, Level::Warning);
            }

            return;
        }

        if let EventV1::Logout = event {
            info!("User {addr:?} received log out event!");
            return;
        }
    }
}

async fn worker(
    addr: SocketAddr,
    subscribed: Arc<RwLock<HashSet<String>>>,
    user_id: String,
    config: &ProtocolConfiguration,
    mut read: WsReader,
    write: &Mutex<WsWriter>,
) {
    loop {
        let result = read.try_next().await;
        let msg = match result {
            Ok(Some(msg)) => msg,
            Ok(None) => {
                warn!("Received a None message!");
                sentry::capture_message("Received a None message!", Level::Warning);
                return;
            }
            Err(e) => {
                use async_tungstenite::tungstenite::Error;
                if !matches!(e, Error::AlreadyClosed | Error::ConnectionClosed) {
                    let err = format!("Error while reading an event from {addr:?}: {e:?}");
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
                if !subscribed.read().await.contains(&channel) {
                    continue;
                }

                EventV1::ChannelStartTyping {
                    id: channel.clone(),
                    user: user_id.clone(),
                }
                .p(channel.clone())
                .await;
            }
            ClientMessage::EndTyping { channel } => {
                if !subscribed.read().await.contains(&channel) {
                    continue;
                }

                EventV1::ChannelStopTyping {
                    id: channel.clone(),
                    user: user_id.clone(),
                }
                .p(channel.clone())
                .await;
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
