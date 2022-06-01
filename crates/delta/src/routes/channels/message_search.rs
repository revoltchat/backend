use revolt_quark::{
    models::{
        message::{BulkMessageResponse, MessageSort},
        User,
    },
    perms, Db, Error, Permission, Ref, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// # Search Parameters
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct OptionsMessageSearch {
    /// Full-text search query
    ///
    /// See [MongoDB documentation](https://docs.mongodb.com/manual/text-search/#-text-operator) for more information.
    #[validate(length(min = 1, max = 64))]
    query: String,

    /// Maximum number of messages to fetch
    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,
    /// Message id before which messages should be fetched
    #[validate(length(min = 26, max = 26))]
    before: Option<String>,
    /// Message id after which messages should be fetched
    #[validate(length(min = 26, max = 26))]
    after: Option<String>,
    /// Message sort direction
    ///
    /// By default, it will be sorted by relevance.
    #[serde(default = "MessageSort::default")]
    sort: MessageSort,
    /// Whether to include user (and member, if server channel) objects
    include_users: Option<bool>,
}

/// # Search for Messages
///
/// This route searches for messages within the given parameters.
#[openapi(tag = "Messaging")]
#[post("/<target>/search", data = "<options>")]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    options: Json<OptionsMessageSearch>,
) -> Result<Json<BulkMessageResponse>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let options = options.into_inner();
    options
        .validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission_and_view_channel(db, Permission::ReadMessageHistory)
        .await?;

    let OptionsMessageSearch {
        query,
        limit,
        before,
        after,
        sort,
        include_users,
    } = options;

    let messages = db
        .search_messages(channel.id(), &query, limit, before, after, sort)
        .await?;

    BulkMessageResponse::transform(db, &channel, messages, include_users)
        .await
        .map(Json)
}
