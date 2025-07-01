use revolt_database::util::permissions::DatabasePermissionQuery;
use revolt_database::{util::reference::Reference, Channel, Database, User};
use revolt_models::v0;
use revolt_permissions::{calculate_server_permissions, ChannelPermission};
use revolt_result::Result;

use crate::util::json::{Json, Validate};
use rocket::State;

/// # Create Channel
///
/// Create a new Text or Voice channel.
#[openapi(tag = "Server Information")]
#[post("/<server>/channels", data = "<data>")]
pub async fn create_server_channel(
    db: &State<Database>,
    user: User,
    server: Reference,
    data: Validate<Json<v0::DataCreateServerChannel>>,
) -> Result<Json<v0::Channel>> {
    let data = data.into_inner().into_inner();

    let mut server = server.as_server(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    calculate_server_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageChannel)?;

    Channel::create_server_channel(db, &mut server, data, true)
        .await
        .map(|channel| channel.into())
        .map(Json)
}
