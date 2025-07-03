use revolt_database::{AdminAuthorization, Database};
use revolt_models::v0::AdminUser;
use revolt_result::Result;
use rocket::{serde::json::Json, State};

/// Get a list of admin users. Any active user may use this endpoint.
/// Typically the client should cache this data.
#[openapi(tag = "Admin")]
#[get("/users")]
pub async fn admin_fetch_users(
    db: &State<Database>,
    auth: AdminAuthorization,
) -> Result<Json<Vec<AdminUser>>> {
    let users = db.admin_user_list().await?;
    let userids: Vec<String> = users.iter().map(|u| u.platform_user_id.clone()).collect();
    let revolt_users = db.fetch_users(&userids).await?;

    let mut resp = vec![];
    for admin_user in users {
        let mut user = AdminUser::from(admin_user);
        user.revolt_user = match revolt_users.iter().find(|ru| ru.id == user.id) {
            Some(some_user) => Some(some_user.clone().into_self(false).await),
            None => None,
        };
        resp.push(user);
    }

    Ok(Json(resp))
}
