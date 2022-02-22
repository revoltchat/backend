use revolt_quark::{
    models::{Channel, User},
    perms, Database, Error, Permission, Ref, Result,
};

use rocket::{serde::json::Json, State};

#[get("/<target>")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<Channel>> {
    let target = target.as_channel(db).await?;

    if perms(&user)
        .channel(&target)
        .calc(db)
        .await
        .can_view_channel()
    {
        Ok(Json(target))
    } else {
        Error::from_permission(Permission::ViewChannel)
    }
}
