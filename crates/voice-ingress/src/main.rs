use std::env;

use redis_kiss::{get_connection, AsyncCommands};

use revolt_database::{events::client::EventV1, util::reference::Reference, Database, DatabaseInfo};
use revolt_models::v0::{PartialUserVoiceState, UserVoiceState};
use rocket::{build, post, routes, serde::json::Json, Config, State};
use rocket_empty::EmptyResponse;
use livekit_protocol::WebhookEvent;

use revolt_result::Result;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    revolt_config::configure!(voice_ingress);

    let database = DatabaseInfo::Auto.connect().await.unwrap();

    let _rocket = build()
        .manage(database)
        .mount("/", routes![ingress])
        .configure(Config { port: 8500, ..Default::default() })
        .ignite()
        .await?
        .launch()
        .await?;

    Ok(())
}


#[post("/", data="<body>")]
async fn ingress(db: &State<Database>, body: Json<WebhookEvent>) -> Result<EmptyResponse> {
    let mut conn = get_connection().await.unwrap();

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

            let channel = Reference::from_unchecked(channel_id.clone())
                .as_channel(db)
                .await?;


            let unique_key = format!("{}-{user_id}", channel.server().unwrap_or_else(|| channel.id()));

            conn.sadd::<_, _, u64>(format!("vc-members-{channel_id}"), user_id).await.unwrap();

            conn.set::<_, _, String>(format!("vc-{unique_key}"), &channel_id).await.unwrap();
            conn.set::<_, _, String>(format!("audio-{unique_key}"), true).await.unwrap();
            conn.set::<_, _, String>(format!("deafened-{unique_key}"), false).await.unwrap();
            conn.set::<_, _, String>(format!("screensharing-{unique_key}"), false).await.unwrap();
            conn.set::<_, _, String>(format!("camera-{unique_key}"), false).await.unwrap();

            let voice_state = UserVoiceState {
                id: user_id.clone(),
                audio: false,
                deafened: false,
                screensharing: false,
                camera: false
            };

            EventV1::VoiceChannelJoin {
                id: channel_id.clone(),
                state: voice_state
            }
            .p(channel_id.clone())
            .await
        },
        "participant_left" => {
            let channel_id = channel_id.unwrap();
            let user_id = user_id.unwrap();

            let channel = Reference::from_unchecked(channel_id.clone())
                .as_channel(db)
                .await?;

            conn.srem::<_, _, u64>(format!("vc-members-{channel_id}"), user_id).await.unwrap();

            let unique_key = format!("{}-{user_id}", channel.server().unwrap_or_else(|| channel.id()));

            conn.del::<_, u64>(format!("vc-{unique_key}")).await.unwrap();

            conn.del::<_, u64>(format!("audio-{unique_key}")).await.unwrap();
            conn.del::<_, u64>(format!("deafened-{unique_key}")).await.unwrap();
            conn.del::<_, u64>(format!("screensharing-{unique_key}")).await.unwrap();
            conn.del::<_, u64>(format!("camera-{unique_key}")).await.unwrap();

            EventV1::VoiceChannelLeave {
                id: channel_id.clone(),
                user: user_id.clone()
            }
            .p(channel_id.clone())
            .await
        },
        "track_published" | "track_unpublished" => {
            let value = body.event == "track_published"; // to avoid duplicating this entire case twice

            let channel_id = channel_id.unwrap();
            let user_id = user_id.unwrap();
            let track = body.track.as_ref().unwrap();

            let user = Reference::from_unchecked(user_id.clone())
                .as_user(db)
                .await?;

            let channel = Reference::from_unchecked(channel_id.clone())
                .as_channel(db)
                .await?;

            let unique_key = if user.bot.is_some() {
                format!("{}-{user_id}", channel.server().unwrap_or_else(|| channel.id()))
            } else {
                user_id.to_string()
            };

            let partial = match track.source {
                /* TrackSource::Unknown */ 0 => {
                    PartialUserVoiceState::default()
                }
                /* TrackSource::Camera */ 1 => {
                    conn.set::<_, _, String>(format!("camera-{unique_key}"), value).await.unwrap();

                    PartialUserVoiceState {
                        camera: Some(value),
                        ..Default::default()
                    }
                }
                /* TrackSource::Microphone */ 2 => {
                    conn.set::<_, _, String>(format!("audio-{unique_key}"), value).await.unwrap();

                    PartialUserVoiceState {
                        audio: Some(value),
                        ..Default::default()
                    }
                },
                /* TrackSource::ScreenShare | TrackSource::ScreenShareAudio */ 3 | 4 => {
                    conn.set::<_, _, String>(format!("screensharing-{unique_key}"), value).await.unwrap();

                    PartialUserVoiceState {
                        screensharing: Some(value),
                        ..Default::default()
                    }
                },
                _ => unreachable!()
            };

            EventV1::UserVoiceStateUpdate {
                id: user_id.clone(),
                channel_id: channel_id.clone(),
                data: partial
            }
            .p(channel_id.clone())
            .await;
        },
        _ => {}
    };

    Ok(EmptyResponse)
}