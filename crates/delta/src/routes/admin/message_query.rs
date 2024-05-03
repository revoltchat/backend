use revolt_quark::{
    models::{
        message::{BulkMessageResponse, MessageQuery},
        User,
    },
    Db, Error, Result,
};
use rocket::serde::json::Json;

/// # Globally Fetch Messages
///
/// This is a privileged route to globally fetch messages.
#[openapi(tag = "Admin")]
#[post("/messages", data = "<data>")]
pub async fn message_query(
    db: &Db,
    user: User,
    data: Json<MessageQuery>,
) -> Result<Json<BulkMessageResponse>> {
    // Must be privileged for this route
    if !user.privileged {
        return Err(Error::NotPrivileged);
    }

    // Fetch data using query
    let data = data.into_inner();
    let messages = db.fetch_messages(data).await?;
    BulkMessageResponse::transform(db, None, messages, Some(true))
        .await
        .map(Json)
}
