use revolt_quark::{
    models::{Member, User},
    perms, Db, Error, Ref, Result,
};
use rocket::serde::json::Json;

#[get("/<target>/members/<member>")]
pub async fn req(db: &Db, user: User, target: Ref, member: Ref) -> Result<Json<Member>> {
    let server = target.as_server(db).await?;
    if !perms(&user)
        .server(&server)
        .calc_server(db)
        .await
        .get_view()
    {
        return Err(Error::NotFound);
    }

    member.as_member(db, &server.id).await.map(Json)
}
