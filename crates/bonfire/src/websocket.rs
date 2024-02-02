use std::net::SocketAddr;

use async_tungstenite::WebSocketStream;
use fred::interfaces::{ClientLike, EventInterface, PubsubInterface};
use futures::{
    channel::oneshot,
    pin_mut, select,
    stream::{SplitSink, SplitStream},
    FutureExt, SinkExt, StreamExt, TryStreamExt,
};
use revolt_presence::{create_session, delete_session};
use revolt_quark::{
    events::{
        client::EventV1,
        server::ClientMessage,
        state::{State, SubscriptionStateChange},
    },
    models::{user::UserHint, User},
    redis_kiss::{PayloadType, REDIS_PAYLOAD_TYPE},
    Database,
};

use async_std::{net::TcpStream, sync::Mutex};

use crate::config::{ProtocolConfiguration, WebsocketHandshakeCallback};

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
        while let Ok(message) = read.try_next().await {
            if let Ok(ClientMessage::Authenticate { token }) =
                config.decode(message.as_ref().unwrap())
            {
                config.set_session_token(token);
                break;
            }
        }
    }

    // Try to authenticate the user.
    let Some(token) = config.get_session_token().as_ref() else {
        return;
    };
    let user = match User::from_token(db, token, UserHint::Any).await {
        Ok(user) => user,
        Err(err) => {
            write.send(config.encode(&err)).await.ok();
            return;
        }
    };
    info!("User {addr:?} authenticated as @{}", user.username);

    // Create local state.
    let mut state = State::from(user);
    let user_id = state.cache.user_id.clone();

    // Notify socket we have authenticated.
    if write
        .send(config.encode(&EventV1::Authenticated))
        .await
        .is_err()
    {
        return;
    }

    // Download required data to local cache and send Ready payload.
    let Ok(ready_payload) = state.generate_ready_payload(db).await else {
        return;
    };
    if write.send(config.encode(&ready_payload)).await.is_err() {
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
        // Create a PubSub connection to poll on.
        let listener = listener(db, &mut state, addr, &config, &write).fuse();
        // Read from WebSocket stream.
        let worker = worker(user_id.clone(), &config, read, &write).fuse();

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
    let Ok(subscriber) = fred::prelude::Builder::default_centralized().build_subscriber_client()
    else {
        return;
    };
    if subscriber.init().await.is_err() {
        return;
    };
    let mut message_rx = subscriber.message_rx();
    loop {
        // Check for state changes for subscriptions.
        match state.apply_state() {
            SubscriptionStateChange::Reset => {
                subscriber.unsubscribe_all().await.unwrap();
                for id in state.iter_subscriptions() {
                    subscriber.subscribe(id).await.unwrap();
                }

                #[cfg(debug_assertions)]
                info!("{addr:?} has reset their subscriptions");
            }
            SubscriptionStateChange::Change { add, remove } => {
                for id in remove {
                    #[cfg(debug_assertions)]
                    info!("{addr:?} unsubscribing from {id}");

                    subscriber.unsubscribe(id).await.unwrap();
                }

                for id in add {
                    #[cfg(debug_assertions)]
                    info!("{addr:?} subscribing to {id}");

                    subscriber.subscribe(id).await.unwrap();
                }
            }
            SubscriptionStateChange::None => {}
        }

        // Handle incoming events.
        let Ok(message) = message_rx.recv().await.map_err(|e| {
            info!("Error while consuming pub/sub messages: {e:?}");
            sentry::capture_error(&e);
        }) else {
            return;
        };
        match *REDIS_PAYLOAD_TYPE {
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

        let Some(mut event) = message
            .value
            .as_str()
            .and_then(|s| serde_json::from_str::<EventV1>(s.as_ref()).ok())
        else {
            warn!("Failed to deserialise an event for {}!", message.channel);
            return;
        };
        let should_send = state.handle_incoming_event_v1(db, &mut event).await;

        if should_send
            && write
                .lock()
                .await
                .send(config.encode(&event))
                .await
                .is_err()
        {
            return;
        }
    }
}

async fn worker(
    user_id: String,
    config: &ProtocolConfiguration,
    mut read: WsReader,
    write: &Mutex<WsWriter>,
) {
    while let Ok(Some(msg)) = read.try_next().await {
        let Ok(payload) = config.decode(&msg) else {
            continue;
        };
        match payload {
            ClientMessage::BeginTyping { channel } => {
                EventV1::ChannelStartTyping {
                    id: channel.clone(),
                    user: user_id.clone(),
                }
                .p(channel.clone())
                .await;
            }
            ClientMessage::EndTyping { channel } => {
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
