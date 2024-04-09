use std::env;

use redis_kiss::{get_connection, AsyncCommands};

use revolt_database::events::client::EventV1;
use rocket::{build, post, routes, serde::json::Json, Config};
use rocket_empty::EmptyResponse;
use livekit_protocol::WebhookEvent;

use revolt_config::Database;
use revolt_result::Result;

#[async_std::main]
async fn main() {
    revolt_config::configure!(voice_ingress);

    let rocket = build()
        .manage(get_connection().await.unwrap())
        .mount("/", routes![ingress])
        .configure(Config { port: 8500, ..Default::default() })
        .launch()
        .await
        .unwrap();
}

#[post("/", data="<body>")]
async fn ingress(body: Json<WebhookEvent>) -> Result<EmptyResponse> {
    let mut redis = get_connection().await.unwrap();

    log::debug!("received event: {body:?}");

    let channel_id = body
        .room
        .as_ref()
        .map(|r| &r.name);

    let user_id = body
        .participant
        .as_ref()
        .map(|r| &r.identity);

    match body.event.as_str() {
        "participant_joined" => {
            let channel_id = channel_id.unwrap();
            let user_id = user_id.unwrap();

            redis.sadd::<_, _, u64>(format!("vc-members-{}", channel_id), user_id).await.unwrap();
            redis.set::<_, _, String>(format!("vc-{}", user_id), &channel_id).await.unwrap();

            EventV1::VoiceChannelJoin {
                id: channel_id.clone(),
                user: user_id.clone()
            }
            .p(channel_id.clone())
            .await
        },
        "participant_left" => {
            let channel_id = channel_id.unwrap();
            let user_id = user_id.unwrap();

            redis.srem::<_, _, u64>(format!("vc-members-{}", channel_id), user_id).await.unwrap();
            redis.del::<_, u64>(format!("vc-{}", user_id)).await.unwrap();

            EventV1::VoiceChannelLeave {
                id: channel_id.clone(),
                user: user_id.clone()
            }
            .p(channel_id.clone())
            .await
        },
        _ => {}
    };

    Ok(EmptyResponse)
}