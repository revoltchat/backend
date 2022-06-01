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

/// # Query Parameters
#[derive(Validate, Serialize, Deserialize, JsonSchema, FromForm)]
pub struct OptionsQueryMessages {
    /// Maximum number of messages to fetch
    ///
    /// For fetching nearby messages, this is \`(limit + 1)\`.
    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,
    /// Message id before which messages should be fetched
    #[validate(length(min = 26, max = 26))]
    before: Option<String>,
    /// Message id after which messages should be fetched
    #[validate(length(min = 26, max = 26))]
    after: Option<String>,
    /// Message sort direction
    sort: Option<MessageSort>,
    /// Message id to search around
    ///
    /// Specifying 'nearby' ignores 'before', 'after' and 'sort'.
    /// It will also take half of limit rounded as the limits to each side.
    /// It also fetches the message ID specified.
    #[validate(length(min = 26, max = 26))]
    nearby: Option<String>,
    /// Whether to include user (and member, if server channel) objects
    include_users: Option<bool>,
}

/// # Fetch Messages
///
/// Fetch multiple messages.
#[openapi(tag = "Messaging")]
#[get("/<target>/messages?<options..>")]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    options: OptionsQueryMessages,
) -> Result<Json<BulkMessageResponse>> {
    options
        .validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if let Some(MessageSort::Relevance) = options.sort {
        return Err(Error::InvalidOperation);
    }

    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission_and_view_channel(db, Permission::ReadMessageHistory)
        .await?;

    let OptionsQueryMessages {
        limit,
        before,
        after,
        sort,
        nearby,
        include_users,
        ..
    } = options;

    let messages = db
        .fetch_messages(channel.id(), limit, before, after, sort, nearby)
        .await?;

    BulkMessageResponse::transform(db, &channel, messages, include_users)
        .await
        .map(Json)
}
