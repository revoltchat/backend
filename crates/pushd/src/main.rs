use amqprs::{
    channel::{BasicConsumeArguments, QueueBindArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
};
use revolt_config::{config, Settings};
use tokio::sync::Notify;

mod consumers;
use consumers::{
    origin::OriginMessageConsumer, outbound::apn::ApnsOutboundConsumer,
    outbound::fcm::FcmOutboundConsumer, outbound::vapid::VapidOutboundConsumer,
};

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let config = config().await;

    // Setup database
    let db = revolt_database::DatabaseInfo::Auto.connect().await.unwrap();
    let authifier: authifier::Database;

    if let Some(client) = match &db {
        revolt_database::Database::Reference(_) => None,
        revolt_database::Database::MongoDb(mongo) => Some(mongo),
    } {
        authifier =
            authifier::Database::MongoDb(authifier::database::MongoDb(client.database("revolt")));
    } else {
        panic!("Mongo is not in use, can't connect via authifier!")
    }

    let mut connections: Vec<Connection> = Vec::new();

    connections.push(
        make_queue_and_consume(
            &config,
            &config.pushd.message_queue,
            OriginMessageConsumer::new(db.clone(), authifier.clone()),
        )
        .await,
    );

    if !config.pushd.apn.pkcs8.is_empty() {
        connections.push(
            make_queue_and_consume(
                &config,
                &config.pushd.apn.queue,
                ApnsOutboundConsumer::new(db.clone()).await.unwrap(),
            )
            .await,
        );
    }

    if !config.pushd.fcm.api_key.is_empty() {
        connections.push(
            make_queue_and_consume(
                &config,
                &config.pushd.fcm.queue,
                FcmOutboundConsumer::new(db.clone()).await.unwrap(),
            )
            .await,
        )
    }

    if !config.pushd.vapid.public_key.is_empty() {
        connections.push(
            make_queue_and_consume(
                &config,
                &config.pushd.fcm.queue,
                VapidOutboundConsumer::new(db.clone()).await.unwrap(),
            )
            .await,
        )
    }

    let guard = Notify::new();
    guard.notified().await;

    for conn in connections {
        conn.close().await.expect("Unable to close connection");
    }
}

async fn make_queue_and_consume<F>(config: &Settings, name: &str, consumer: F) -> Connection
where
    F: AsyncConsumer + Send + 'static,
{
    let connection = Connection::open(&OpenConnectionArguments::new(
        "127.0.0.1",
        5672,
        "guest",
        "guest",
    ))
    .await
    .unwrap();

    let channel = connection.open_channel(None).await.unwrap();

    let args = QueueDeclareArguments::new(name).durable(true).finish();
    let (queue_name, _, _) = channel.queue_declare(args).await.unwrap().unwrap();

    channel
        .queue_bind(QueueBindArguments::new(
            &queue_name,
            &config.pushd.exchange,
            name,
        ))
        .await
        .unwrap();

    let args = BasicConsumeArguments::new(name, "basic_consumer")
        .manual_ack(false)
        .finish();

    channel.basic_consume(consumer, args).await.unwrap();
    connection
}
