//! Open a direct message with another user

use revolt_quark::{
    models::{Channel, User},
    perms, Database, Error, Ref, Result, UserPermission,
};

use rocket::{serde::json::Json, State};
use ulid::Ulid;

#[get("/<target>/dm")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<Channel>> {
    let target = target.as_user(db).await?;

    if let Ok(channel) = db.find_direct_message_channel(&user.id, &target.id).await {
        Ok(Json(channel))
    } else if perms(&user)
        .user(&target)
        .calc_user(db)
        .await
        .get_send_message()
    {
        let new_channel = Channel::DirectMessage {
            id: Ulid::new().to_string(),
            active: false,
            recipients: vec![user.id.clone(), target.id.clone()],
            last_message_id: None,
        };

        db.insert_channel(&new_channel).await?;
        Ok(Json(new_channel))
    } else {
        Error::from_permission(UserPermission::SendMessage)
    }
}
