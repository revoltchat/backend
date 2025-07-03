use revolt_database::{util::reference::Reference, AdminAuthorization, Database};
use revolt_models::v0::{self};
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};

use crate::routes::admin::util::{flatten_authorized_user, user_has_permission};

/// Edit an object note. Requires ObjectNotes permission.
#[openapi(tag = "Admin")]
#[get("/notes/<object>")]
pub async fn admin_object_get_note(
    db: &State<Database>,
    auth: AdminAuthorization,
    object: Reference,
) -> Result<Json<v0::AdminObjectNote>> {
    let user = flatten_authorized_user(&auth);
    if !user_has_permission(user, v0::AdminUserPermissionFlags::ObjectNotes) {
        return Err(create_error!(NotFound));
    }

    // Unfortunately due to how our system works, we can't limit users to editing objects they have permissions for (eg users, servers, etc.).
    // Hence it being tied to its own permission
    Ok(Json(db.admin_note_fetch(&object.id).await?.into()))
}
