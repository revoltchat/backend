use std::collections::HashSet;

use crate::database::*;
use crate::util::result::{Error, Result};

use futures::{StreamExt, try_join};
use mongodb::{
    bson::{doc, from_document},
    options::FindOptions,
};
use rocket::request::Form;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, FromFormValue)]
pub enum Sort {
    Latest,
    Oldest,
}

#[derive(Validate, Serialize, Deserialize, FromForm)]
pub struct Options {
    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,
    #[validate(length(min = 26, max = 26))]
    before: Option<String>,
    #[validate(length(min = 26, max = 26))]
    after: Option<String>,
    sort: Option<Sort>,
    // Specifying 'nearby' ignores 'before', 'after' and 'sort'.
    // It will also take half of limit rounded as the limits to each side.
    // It also fetches the message ID specified.
    #[validate(length(min = 26, max = 26))]
    nearby: Option<String>,
    include_users: Option<bool>,
}

#[get("/<target>/messages?<options..>")]
pub async fn req(user: User, target: Ref, options: Form<Options>) -> Result<JsonValue> {
    options
        .validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let target = target.fetch_channel().await?;
    target.has_messaging()?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel()
        .await?;
    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    let mut messages = vec![];

    let collection = get_collection("messages");
    let limit = options.limit.unwrap_or(50);
    let channel = target.id();
    if let Some(nearby) = &options.nearby {
        let mut cursors = try_join!(
            collection.find(
                doc! {
                    "channel": channel,
                    "_id": {
                        "$gte": &nearby
                    }
                },
                FindOptions::builder()
                    .limit(limit / 2 + 1)
                    .sort(doc! {
                        "_id": 1
                    })
                    .build(),
            ),
            collection.find(
                doc! {
                    "channel": channel,
                    "_id": {
                        "$lt": &nearby
                    }
                },
                FindOptions::builder()
                    .limit(limit / 2)
                    .sort(doc! {
                        "_id": -1
                    })
                    .build(),
            )
        )
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "messages",
        })?;

        while let Some(result) = cursors.0.next().await {
            if let Ok(doc) = result {
                messages.push(
                    from_document::<Message>(doc).map_err(|_| Error::DatabaseError {
                        operation: "from_document",
                        with: "message",
                    })?,
                );
            }
        }

        while let Some(result) = cursors.1.next().await {
            if let Ok(doc) = result {
                messages.push(
                    from_document::<Message>(doc).map_err(|_| Error::DatabaseError {
                        operation: "from_document",
                        with: "message",
                    })?,
                );
            }
        }
    } else {
        let mut query = doc! { "channel": target.id() };
        if let Some(before) = &options.before {
            query.insert("_id", doc! { "$lt": before });
        }

        if let Some(after) = &options.after {
            query.insert("_id", doc! { "$gt": after });
        }

        let sort: i32 = if let Sort::Latest = options.sort.as_ref().unwrap_or_else(|| &Sort::Latest) {
            -1
        } else {
            1
        };

        let mut cursor = collection
            .find(
                query,
                FindOptions::builder()
                    .limit(limit)
                    .sort(doc! {
                        "_id": sort
                    })
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: "messages",
            })?;

        while let Some(result) = cursor.next().await {
            if let Ok(doc) = result {
                messages.push(
                    from_document::<Message>(doc).map_err(|_| Error::DatabaseError {
                        operation: "from_document",
                        with: "message",
                    })?,
                );
            }
        }
    }

    if options.include_users.unwrap_or_else(|| false) {
        let mut ids = HashSet::new();
        for message in &messages {
            ids.insert(message.author.clone());
        }

        ids.remove(&user.id);
        let user_ids = ids.into_iter().collect();
        let users = user.fetch_multiple_users(user_ids).await?;

        if let Channel::TextChannel { server, .. } = target {
            Ok(json!({
                "messages": messages,
                "users": users,
                "members": Server::fetch_members(&server).await?
            }))
        } else {
            Ok(json!({
                "messages": messages,
                "users": users,
            }))
        }
    } else {
        Ok(json!(messages))
    }
}
