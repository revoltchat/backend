use revolt_quark::{
    models::{
        message::{BulkMessageResponse, MessageSort},
        User,
    },
    perms, Db, Error, Ref, Result,
};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, FromForm)]
pub struct Options {
    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,
    #[validate(length(min = 26, max = 26))]
    before: Option<String>,
    #[validate(length(min = 26, max = 26))]
    after: Option<String>,
    sort: Option<MessageSort>,
    // Specifying 'nearby' ignores 'before', 'after' and 'sort'.
    // It will also take half of limit rounded as the limits to each side.
    // It also fetches the message ID specified.
    #[validate(length(min = 26, max = 26))]
    nearby: Option<String>,
    include_users: Option<bool>,
}

#[get("/<target>/messages?<options..>")]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    options: Options,
) -> Result<Json<BulkMessageResponse>> {
    options
        .validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if let Some(MessageSort::Relevance) = options.sort {
        return Err(Error::InvalidOperation);
    }

    let channel = target.as_channel(db).await?;
    if !perms(&user)
        .channel(&channel)
        .calc_channel(db)
        .await
        .get_view()
    {
        return Err(Error::NotFound);
    }

    let Options {
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
