use revolt_config::config;

pub mod event_stream;

/// Create a lapin client
pub async fn create_client() -> lapin::Connection {
    let config = config().await;

    lapin::Connection::connect(
        &format!(
            "amqp://{}:{}@{}:{}/%2f",
            config.rabbit.username, config.rabbit.password, config.rabbit.host, config.rabbit.port
        ),
        Default::default(),
    )
    .await
    .unwrap()
}
