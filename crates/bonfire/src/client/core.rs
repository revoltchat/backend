use async_std::{net::TcpStream, sync::Mutex};
use async_tungstenite::WebSocketStream;
use futures::{join, SinkExt, StreamExt, TryStreamExt};
use revolt_config::report_internal_error;
use revolt_database::{
    events::{client::EventV1, server::ClientMessage},
    iso8601_timestamp::Timestamp,
    Database, User, UserHint,
};
use revolt_presence::{create_session, delete_session};
use revolt_result::create_error;

use crate::{
    client::{
        subscriber::client_subscriber,
        worker::{client_worker, WorkerRef},
    },
    config::ProtocolConfiguration,
    events::state::State,
};

/// Core event loop of gateway clients
pub async fn client_core(
    db: &'static Database,
    ws: WebSocketStream<TcpStream>,
    mut config: ProtocolConfiguration,
) {
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
            .send(config.encode(&EventV1::Error {
                data: create_error!(InvalidSession),
            }))
            .await
            .ok();
        return;
    };

    let (user, session_id) = match User::from_token(db, token, UserHint::Any).await {
        Ok(user) => user,
        Err(err) => {
            write
                .send(config.encode(&EventV1::Error { data: err }))
                .await
                .ok();
            return;
        }
    };

    info!(
        "Authenticated user {}#{}",
        user.username, user.discriminator
    );

    db.update_session_last_seen(&session_id, Timestamp::now_utc())
        .await
        .ok();

    // Create local state.
    let mut state = State::from(user, session_id);
    let user_id = state.cache.user_id.clone();

    // Notify socket we have authenticated.
    if report_internal_error!(write.send(config.encode(&EventV1::Authenticated)).await).is_err() {
        return;
    }

    // Download required data to local cache and send Ready payload.
    let ready_payload = match report_internal_error!(
        state
            .generate_ready_payload(db, config.get_ready_payload_fields())
            .await
    ) {
        Ok(ready_payload) => ready_payload,
        Err(_) => return,
    };

    if report_internal_error!(write.send(config.encode(&ready_payload)).await).is_err() {
        return;
    }

    // Create presence session.
    let (first_session, session_id) = create_session(&user_id, 0).await;

    // If this was the first session, notify other users that we just went online.
    if first_session {
        state.broadcast_presence_change(true).await;
    }

    {
        let worker_ref = WorkerRef::from(&state);
        let write = Mutex::new(write);
        let (reload, reloaded) = async_channel::bounded(1);
        let (cancel_1, cancelled_1) = async_channel::bounded(1);
        let (cancel_2, cancelled_2) = async_channel::bounded(1);

        join!(
            async {
                client_subscriber(&write, cancelled_1, reloaded, &config, db, &mut state).await;
                cancel_2.send(()).await.ok();
            },
            async {
                client_worker(read, &write, cancelled_2, reload, &config, worker_ref).await;
                cancel_1.send(()).await.ok();
            }
        );
    }

    // Clean up presence session.
    let last_session = delete_session(&user_id, session_id).await;

    // If this was the last session, notify other users that we just went offline.
    if last_session {
        state.broadcast_presence_change(false).await;
    }
}
