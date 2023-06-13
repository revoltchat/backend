use std::net::SocketAddr;

use async_tungstenite::WebSocketStream;
use futures::{channel::oneshot, pin_mut, select, FutureExt, SinkExt, StreamExt, TryStreamExt};
use revolt_presence::{create_session, delete_session};
use revolt_quark::{
    events::{
        client::EventV1,
        server::ClientMessage,
        state::{State, SubscriptionStateChange},
    },
    models::{user::UserHint, User},
    redis_kiss, Database, Error as QuarkError,
};

use async_std::{net::TcpStream, sync::Mutex, task};

use crate::config::{ProtocolConfiguration, WebsocketHandshakeCallback};

/// Spawn a new WebSocket client worker given access to the database,
/// the relevant TCP stream and the remote address of the client.
pub fn spawn_client(db: &'static Database, stream: TcpStream, addr: SocketAddr) {
    // Spawn a new Async task to work on.
    task::spawn(async move {
        info!("User connected from {addr:?}");

        // Upgrade the TCP connection to a WebSocket connection.
        // In this process, we also parse any additional parameters given.
        // e.g. wss://example.com?format=json&version=1
        let (sender, receiver) = oneshot::channel();
        if let Ok(mut ws) = async_tungstenite::accept_hdr_async_with_config(
            stream,
            WebsocketHandshakeCallback::from(sender),
            None,
        )
        .await
        {
            // Verify we've received a valid config, otherwise we should just drop the connection.
            if let Ok(mut config) = receiver.await {
                info!(
                    "User {addr:?} provided protocol configuration (version = {}, format = {:?})",
                    config.get_protocol_version(),
                    config.get_protocol_format()
                );

                // Split the socket for simultaneously read and write.
                // let (write, mut read) = ws.split();
                // let write = Mutex::new(write);

                // If the user has not provided authentication, request information.
                if config.get_session_token().is_none() {
                    'outer: while let Ok(message) = ws.try_next().await {
                        if let Ok(ClientMessage::Authenticate { token }) =
                            config.decode(message.as_ref().unwrap())
                        {
                            config.set_session_token(token);
                            break 'outer;
                        }
                    }
                }

                // Reuinite the websocket to make it easier to pass to additional functions.
                // Unwrap *should* be okay because we can guaruntee that they are from the same
                // socket.
                // let ws = read.reunite(write.into_inner()).unwrap();

                // Try to authenticate the user.
                if let Some(token) = config.get_session_token().as_ref() {
                    match User::from_token(db, token, UserHint::Any).await {
                        Ok(user) => {
                            info!("User {addr:?} authenticated as @{}", user.username);
                            // We don't really care about these errors, but it's convenient to be
                            // able to return them with `?`
                            let _ = setup_user(db, &mut ws, addr, user, config).await;
                        }
                        Err(err) => {
                            ws.send(config.encode(&err)).await.ok();
                        }
                    }
                }
            }

            // * Disconnect the WebSocket if it isn't already.
            /*ws.close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: std::borrow::Cow::from(""),
            }))
            .await
            .unwrap();*/
        }

        info!("User disconnected from {addr:?}");
    });
}

/// Setup a user given the relevant socket and DB information, config, and found user
async fn setup_user(
    db: &'static Database,
    mut ws: &mut WebSocketStream<TcpStream>,
    addr: SocketAddr,
    user: User,
    config: ProtocolConfiguration,
) -> Result<(), QuarkError> {
    // Split the socket for simultaneously read and write.
    let (write, read) = ws.split();
    let write = Mutex::new(write);

    // Create local state.
    let mut state = State::from(user);
    let user_id = state.cache.user_id.clone();

    // Create presence session.
    let (first_session, session_id) = create_session(&user_id, 0).await;

    // Notify socket we have authenticated.
    write
        .lock()
        .await
        .send(config.encode(&EventV1::Authenticated))
        .await
        .ok();

    // Download required data to local cache and send Ready payload.
    let ready_payload = state.generate_ready_payload(db).await?;
    write
        .lock()
        .await
        .send(config.encode(&ready_payload))
        .await
        .ok();

    // If this was the first session, notify other users that we just went online.
    if first_session {
        state.broadcast_presence_change(true).await;
    }

    // Reuinite the websocket to make it easier to pass to additional functions.
    // Unwrap *should* be okay because we can guaruntee that they are from the same
    // socket.
    ws = read.reunite(write.into_inner()).unwrap();

    // Create and poll pubsub connection
    poll_pubsub_connection(db, &mut ws, addr, config, &user_id, &mut state).await;

    // Clean up presence session.
    let last_session = delete_session(&user_id, session_id).await;

    // If this was the last session, notify other users that we just went offline.
    if last_session {
        state.broadcast_presence_change(false).await;
    }
    Ok(())
}

async fn poll_pubsub_connection(
    db: &'static Database,
    ws: &mut WebSocketStream<TcpStream>,
    addr: SocketAddr,
    config: ProtocolConfiguration,
    user_id: &str,
    state: &mut State,
) {
    // Split the socket for simultaneously read and write.
    let (write, mut read) = ws.split();
    let write = Mutex::new(write);

    let listener = async {
        let mut conn = match redis_kiss::open_pubsub_connection().await {
            Ok(conn) => conn,
            // Ignore error, just return early.
            Err(_) => return,
        };
        loop {
            // Check for state changes for subscriptions.
            match state.apply_state() {
                SubscriptionStateChange::Reset => {
                    for id in state.iter_subscriptions() {
                        conn.subscribe(id).await.unwrap();
                    }

                    #[cfg(debug_assertions)]
                    info!("{addr:?} has reset their subscriptions");
                }
                SubscriptionStateChange::Change { add, remove } => {
                    for id in remove {
                        #[cfg(debug_assertions)]
                        info!("{addr:?} unsubscribing from {id}");

                        conn.unsubscribe(id).await.unwrap();
                    }

                    for id in add {
                        #[cfg(debug_assertions)]
                        info!("{addr:?} subscribing to {id}");

                        conn.subscribe(id).await.unwrap();
                    }
                }
                SubscriptionStateChange::None => {}
            }

            // * Debug logging of current subscriptions.
            /*#[cfg(debug_assertions)]
            info!(
                "User {addr:?} is subscribed to {:?}",
                state
                    .iter_subscriptions()
                    .collect::<Vec<&String>>()
            );*/

            // Handle incoming events.
            match conn.on_message().next().await.map(|item| {
                (
                    item.get_channel_name().to_string(),
                    redis_kiss::decode_payload::<EventV1>(&item),
                )
            }) {
                Some((channel, item)) => {
                    if let Ok(mut event) = item {
                        if state.handle_incoming_event_v1(db, &mut event).await
                            && write
                                .lock()
                                .await
                                .send(config.encode(&event))
                                .await
                                .is_err()
                        {
                            break;
                        }
                    } else {
                        warn!("Failed to deserialise an event for {channel}!");
                    }
                }
                // No more data, assume we disconnected or otherwise
                // something bad occurred, so disconnect user.
                None => break,
            }
        }
    }
    .fuse();

    // Read from WebSocket stream.
    let worker = async {
        while let Ok(Some(msg)) = read.try_next().await {
            if let Ok(payload) = config.decode(&msg) {
                match payload {
                    ClientMessage::BeginTyping { channel } => {
                        EventV1::ChannelStartTyping {
                            id: channel.clone(),
                            user: user_id.to_string(),
                        }
                        .p(channel.clone())
                        .await;
                    }
                    ClientMessage::EndTyping { channel } => {
                        EventV1::ChannelStopTyping {
                            id: channel.clone(),
                            user: user_id.to_string(),
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
    }
    .fuse();

    // Pin both tasks.
    pin_mut!(listener, worker);

    // Wait for either disconnect or for listener to die.
    select!(
        () = listener => {},
        () = worker => {}
    );
}
