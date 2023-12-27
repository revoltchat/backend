use revolt_quark::models::server_member::MemberResponse;
use revolt_quark::{models::User, perms, Db, Ref, Result};
use rocket::serde::json::Json;
/// # Fetch Member
///
/// Retrieve a member.
#[openapi(tag = "Server Members")]
#[get("/<target>/members/<member>?<roles>")]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    member: Ref,
    roles: Option<bool>,
) -> Result<Json<MemberResponse>> {
    let server = target.as_server(db).await?;
    perms(&user).server(&server).calc(db).await?;

    let member_response: MemberResponse = match roles {
        Some(true) => member.as_member_with_roles(db, &server.id).await?.into(),
        _ => member.as_member(db, &server.id).await?.into(),
    };

    Ok(Json(member_response))
}
