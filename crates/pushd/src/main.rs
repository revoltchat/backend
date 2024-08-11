use amqprs::{
    callbacks::DefaultChannelCallback,
    channel::{BasicConsumeArguments, QueueBindArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
};
use revolt_config::config;
use tokio::sync::Notify;

mod consumers;

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

    let connection = Connection::open(&OpenConnectionArguments::new(
        "127.0.0.1",
        5672,
        "guest",
        "guest",
    ))
    .await
    .unwrap();

    let channel = connection.open_channel(None).await.unwrap();

    let args = QueueDeclareArguments::new("notifications.ingest.message")
        .durable(true)
        .finish();

    let (queue_name, _, _) = channel.queue_declare(args).await.unwrap().unwrap();

    channel
        .queue_bind(QueueBindArguments::new(
            &queue_name,
            "revolt.notifications",
            "notifications",
        ))
        .await
        .unwrap();

    let args = BasicConsumeArguments::new(&queue_name, "basic_consumer")
        .manual_ack(true)
        .finish();

    channel
        .basic_consume(
            consumers::origin::OriginConsumer::new(db.clone(), authifier.clone(), config),
            args,
        )
        .await
        .unwrap();

    let guard = Notify::new();
    guard.notified().await;
}
