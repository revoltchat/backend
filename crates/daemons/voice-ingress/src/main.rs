use std::env;

use amqprs::{
    channel::ExchangeDeclareArguments,
    connection::{Connection, OpenConnectionArguments},
};
use revolt_config::config;
use revolt_database::DatabaseInfo;
use revolt_database::{voice::VoiceClient, AMQP};
use revolt_result::Result;
use rocket::{build, routes, Config};
use std::net::Ipv4Addr;

mod api;
mod guard;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    revolt_config::configure!(voice_ingress);

    let config = config().await;

    let database = DatabaseInfo::Auto.connect().await.unwrap();
    let voice_client = VoiceClient::from_revolt_config().await;

    let connection = Connection::open(&OpenConnectionArguments::new(
        &config.rabbit.host,
        config.rabbit.port,
        &config.rabbit.username,
        &config.rabbit.password,
    ))
    .await
    .expect("Failed to connect to RabbitMQ");

    let channel = connection
        .open_channel(None)
        .await
        .expect("Failed to open RabbitMQ channel");

    channel
        .exchange_declare(
            ExchangeDeclareArguments::new(&config.pushd.exchange, "direct")
                .durable(true)
                .finish(),
        )
        .await
        .expect("Failed to declare exchange");

    let amqp = AMQP::new(connection, channel);

    let _rocket = build()
        .manage(database)
        .manage(voice_client)
        .manage(amqp)
        .mount("/", routes![api::ingress])
        .configure(Config {
            port: 8500,
            address: Ipv4Addr::new(0, 0, 0, 0).into(),
            ..Default::default()
        })
        .ignite()
        .await?
        .launch()
        .await?;

    Ok(())
}
