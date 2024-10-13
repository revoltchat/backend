use std::env;
use fastwebsockets::upgrade::{IncomingUpgrade, upgrade};
use fastwebsockets::{WebSocket, WebSocketError};
use http_body_util::Empty;
use hyper::server::conn::http1;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use hyper::service::service_fn;

use tokio::net::TcpListener;
use revolt_presence::clear_region;
use crate::config::{ProtocolConfiguration, ProtocolFormat};
use crate::websocket::client;

#[macro_use]
extern crate log;

pub mod config;
pub mod events;

mod database;
mod websocket;

#[tokio::main]
async fn main() {
    // Configure requirements for Bonfire.
    revolt_config::configure!(events);
    database::connect().await;

    // Clean up the current region information.
    clear_region(None).await;

    // Setup a TCP listener to accept WebSocket connections on.
    // By default, we bind to port 14703 on all interfaces.
    let bind = env::var("HOST").unwrap_or_else(|_| "0.0.0.0:14703".into());
    info!("Listening on host {bind}");
    let try_socket = TcpListener::bind(bind).await;
    let listener = try_socket.expect("Failed to bind");

    // Start accepting new connections and spawn a client for each connection.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::task::spawn(async move {
            let io = hyper_util::rt::TokioIo::new(stream);
            let conn_fut = http1::Builder::new()
                .serve_connection(io, service_fn(server_upgrade))
                .with_upgrades();
            info!("User connected from {addr:?}");
            conn_fut.await.unwrap();
            info!("User disconnected from {addr:?}");
        });
    }
}

async fn server_upgrade(mut req: Request<Incoming>) -> Result<Response<Empty<Bytes>>, WebSocketError> {
    // Take and parse query parameters from the URI.
    let query = req.uri().query().unwrap_or_default();
    let params = querystring::querify(query);

    // Set default values for the protocol.
    let mut protocol_version = 1;
    let mut format = ProtocolFormat::Json;
    let mut session_token = None;

    // Parse and map parameters from key-value to known variables.
    for (key, value) in params {
        match key {
            "version" => {
                if let Ok(version) = value.parse() {
                    protocol_version = version;
                }
            }
            "format" => match value {
                "json" => format = ProtocolFormat::Json,
                "msgpack" => format = ProtocolFormat::Msgpack,
                _ => {}
            },
            "token" => session_token = Some(value.into()),
            _ => {}
        }
    }

    let (response, fut) = upgrade(&mut req)?;

    tokio::task::spawn(async move {
       client(database::get_db(), fut, ProtocolConfiguration {
           protocol_version,
           format,
           session_token
       }).await;
    });

    Ok(response)
}
