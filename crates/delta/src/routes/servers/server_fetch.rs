use revolt_quark::{
    models::{Channel, Server, User},
    perms, Db, Ref, Result,
};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

/// # Query Parameters
#[derive(Deserialize, JsonSchema, FromForm)]
pub struct OptionsFetchServer {
    /// Whether to include channels
    include_channels: Option<bool>,
}

/// # Fetch server route response
#[derive(Serialize, JsonSchema)]
#[serde(untagged)]
pub enum FetchServerResponse {
    JustServer(Server),
    ServerWithChannels {
        #[serde(flatten)]
        server: Server,
        channels: Vec<Channel>,
    },
}

/// # Fetch Server
///
/// Fetch a server by its id.
#[openapi(tag = "Server Information")]
#[get("/<target>?<options..>")]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    options: OptionsFetchServer,
) -> Result<Json<FetchServerResponse>> {
    let server = target.as_server(db).await?;
    let mut perms = perms(&user).server(&server);
    perms.calc(db).await?;

    if let Some(true) = options.include_channels {
        let all_channels = db.fetch_channels(&server.channels).await?;
        let mut visible_channels = vec![];

        for channel in all_channels {
            if perms
                .clone()
                .channel(&channel)
                .calc(db)
                .await?
                .can_view_channel()
            {
                visible_channels.push(channel);
            }
        }

        Ok(Json(FetchServerResponse::ServerWithChannels {
            server,
            channels: visible_channels,
        }))
    } else {
        Ok(Json(FetchServerResponse::JustServer(server)))
    }
}
