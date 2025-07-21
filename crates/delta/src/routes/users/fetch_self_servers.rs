use revolt_database::{util::{oauth2::{scopes, OAuth2Scoped}, permissions::DatabasePermissionQuery}, Database, User};
use revolt_models::v0;
use revolt_permissions::{calculate_channel_permissions, ChannelPermission};
use revolt_result::Result;
use rocket::{serde::json::Json, State};

/// # Fetch current user's servers
///
/// Retrieve all servers the current user is in.
#[openapi(tag = "User Information")]
#[get("/@me/servers?<options..>")]
pub async fn fetch_self_servers(
    db: &State<Database>,
    _oauth2_scope: OAuth2Scoped<scopes::ReadServers>,
    user: User,
    options: v0::OptionsFetchServer,
) -> Result<Json<Vec<v0::FetchServerResponse>>> {
    let members = db.fetch_all_memberships(&user.id).await?;

    let server_ids = members
        .iter()
        .map(|x| x.id.server.clone())
        .collect::<Vec<_>>();

    let mut servers: Vec<v0::FetchServerResponse> = Vec::new();

    for server in db.fetch_servers(&server_ids).await? {
        if let Some(true) = options.include_channels {
            let query = DatabasePermissionQuery::new(db, &user).server(&server);

            let all_channels = db.fetch_channels(&server.channels).await?;
            let mut visible_channels: Vec<v0::Channel> = vec![];

            for channel in all_channels {
                let mut channel_query = query.clone().channel(&channel);
                if calculate_channel_permissions(&mut channel_query)
                    .await
                    .has_channel_permission(ChannelPermission::ViewChannel)
                {
                    visible_channels.push(channel.into());
                }
            }

            servers.push(v0::FetchServerResponse::ServerWithChannels {
                server: server.into(),
                channels: visible_channels,
            });
        } else {
            servers.push(v0::FetchServerResponse::JustServer(server.into()))
        }
    }

    Ok(Json(servers))
}