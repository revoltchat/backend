use std::env;


use livekit_protocol::TrackType;
use livekit_api::{access_token::TokenVerifier, webhooks::WebhookReceiver};
use revolt_database::{
    events::client::EventV1, util::reference::Reference, Database, DatabaseInfo,
    voice::{create_voice_state, delete_voice_state, update_voice_state_tracks, VoiceClient}
};
use rocket::{build, http::Status, post, request::{FromRequest, Outcome}, routes, Config, Request, State};
use rocket_empty::EmptyResponse;
use std::net::Ipv4Addr;
use revolt_result::{create_error, Error, Result, ToRevoltError};

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    revolt_config::configure!(voice_ingress);

    let database = DatabaseInfo::Auto.connect().await.unwrap();
    let voice_client = VoiceClient::from_revolt_config().await;
    let _rocket = build()
        .manage(database)
        .manage(voice_client)
        .mount("/", routes![ingress])
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

struct AuthHeader<'a>(&'a str);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthHeader<'r> {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get("Authorization").next() {
            Some(token) => Outcome::Success(Self(token)),
            None => Outcome::Error((Status::Unauthorized, create_error!(NotAuthenticated)))
        }
    }
}

impl std::ops::Deref for AuthHeader<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[post("/<node>", data = "<body>")]
async fn ingress(db: &State<Database>, node: &str, voice_client: &State<VoiceClient>, auth_header: AuthHeader<'_>, body: &str) -> Result<EmptyResponse> {
    log::debug!("received event: {body:?}");

    let config = revolt_config::config().await;

    let node_info = config.api.livekit.nodes.get(node)
        .ok_or_else(|| create_error!(NotAuthenticated))?;

    let webhook_receiver = WebhookReceiver::new(TokenVerifier::with_api_key(&node_info.key, &node_info.secret));
    let event = webhook_receiver.receive(body, &auth_header).to_internal_error()?;

    let channel_id = event.room.as_ref().map(|r| &r.name);
    let user_id = event.participant.as_ref().map(|r| &r.identity);

    match event.event.as_str() {
        "participant_joined" => {
            let channel_id = channel_id.to_internal_error()?;
            let user_id = user_id.to_internal_error()?;

            let channel = Reference::from_unchecked(channel_id.clone())
                .as_channel(db)
                .await?;

            let voice_state = create_voice_state(channel_id, channel.server(), user_id).await?;

            EventV1::VoiceChannelJoin {
                id: channel_id.clone(),
                state: voice_state,
            }
            .p(channel_id.clone())
            .await
        }
        "participant_left" => {
            let channel_id = channel_id.to_internal_error()?;
            let user_id = user_id.to_internal_error()?;

            let channel = Reference::from_unchecked(channel_id.clone())
                .as_channel(db)
                .await?;

            delete_voice_state(channel_id, channel.server(), user_id).await?;

            EventV1::VoiceChannelLeave {
                id: channel_id.clone(),
                user: user_id.clone(),
            }
            .p(channel_id.clone())
            .await
        }
        "track_published" | "track_unpublished" => {
            let channel_id = channel_id.to_internal_error()?;
            let user_id = user_id.to_internal_error()?;
            let track = event.track.as_ref().to_internal_error()?;

            let channel = Reference::from_unchecked(channel_id.clone())
                .as_channel(db)
                .await?;

            // remove the user if they try publish a video larger than 1080x720 or they publish data
            // TODO: move to config
            if event.event == "track_published"
                && (
                    // handle any size which goes over the limit of "1080x720" to stop people from making too tall or too wide and bypassing the limit
                    (track.r#type == TrackType::Video as i32 && (track.width * track.height) >= (1080 * 720))
                    | (track.r#type == TrackType::Data as i32)
                )
            {
                voice_client.remove_user(node, user_id, channel_id).await?;
                delete_voice_state(channel_id, channel.server(), user_id).await?;
            }

            let partial = update_voice_state_tracks(
                channel_id,
                channel.server(),
                user_id,
                event.event == "track_published",  // to avoid duplicating this entire case twice
                track.source
            ).await?;

            EventV1::UserVoiceStateUpdate {
                id: user_id.clone(),
                channel_id: channel_id.clone(),
                data: partial
            }
            .p(channel_id.clone())
            .await;
        }
        _ => {}
    };

    Ok(EmptyResponse)
}
