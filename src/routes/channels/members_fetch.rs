use revolt_quark::{
    models::{Channel, User},
    perms, Db, Error, Ref, Result,
};

use rocket::serde::json::Json;

#[get("/<target>/members")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Vec<User>>> {
    let channel = target.as_channel(db).await?;
    if !perms(&user)
        .channel(&channel)
        .calc_channel(db)
        .await
        .get_view()
    {
        return Err(Error::NotFound);
    }

    if let Channel::Group { recipients, .. } = channel {
        Ok(Json(
            db.fetch_users(&recipients)
                .await?
                .into_iter()
                .map(|x| x.with_relationship(&user))
                .collect::<Vec<User>>(),
        ))
    } else {
        Err(Error::InvalidOperation)
    }
}
