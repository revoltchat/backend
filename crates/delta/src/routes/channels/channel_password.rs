use revolt_quark::{
    models::{Channel, User},
    perms, Database, Error, Permission, Ref, Result,
};
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use validator::Validate;
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataChannelPassword {
    #[validate(length(min = 1))]
    pub password: Option<String>,
}

/// # Fetch Channel
///
/// Fetch channel and check if password matches
#[openapi(tag = "Channel Information")]
#[post("/<target>/password", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    user: User,
    target: Ref,
    data: Json<DataChannelPassword>,
) -> Result<Json<Channel>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission(db, Permission::ViewChannel)
        .await?;

    if let Channel::TextChannel { password, .. } = &channel {
        if data.password != *password {
            return Err(Error::InvalidProperty);
        }

        return Ok(Json(channel));
    } else {
        return Err(Error::InvalidOperation);
    }
}
