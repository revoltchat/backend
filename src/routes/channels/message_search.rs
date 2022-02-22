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

#[derive(Validate, Serialize, Deserialize, FromForm)]
pub struct Options {
    #[validate(length(min = 1, max = 64))]
    query: String,

    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,
    #[validate(length(min = 26, max = 26))]
    before: Option<String>,
    #[validate(length(min = 26, max = 26))]
    after: Option<String>,
    #[serde(default = "MessageSort::default")]
    sort: MessageSort,
    include_users: Option<bool>,
}

#[post("/<target>/search", data = "<options>")]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    options: Json<Options>,
) -> Result<Json<BulkMessageResponse>> {
    let options = options.into_inner();
    options
        .validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission_and_view_channel(db, Permission::ReadMessageHistory)
        .await?;

    let Options {
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
