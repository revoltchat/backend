use std::env;

use revolt_database::voice::VoiceClient;
use revolt_database::DatabaseInfo;
use revolt_result::Result;
use rocket::{build, routes, Config};
use std::net::Ipv4Addr;

mod api;
mod guard;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    revolt_config::configure!(voice_ingress);

    let database = DatabaseInfo::Auto.connect().await.unwrap();
    let voice_client = VoiceClient::from_revolt_config().await;
    let _rocket = build()
        .manage(database)
        .manage(voice_client)
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
