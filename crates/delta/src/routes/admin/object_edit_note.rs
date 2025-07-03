use revolt_database::{util::reference::Reference, AdminAuthorization, AdminObjectNote, Database};
use revolt_models::v0::{self};
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};
use rocket_empty::EmptyResponse;

use crate::routes::admin::util::{flatten_authorized_user, user_has_permission};

/// Edit an object note. Requires ObjectNotes permission.
#[openapi(tag = "Admin")]
#[post("/notes/<object>", data = "<body>")]
pub async fn admin_object_edit_note(
    db: &State<Database>,
    auth: AdminAuthorization,
    object: Reference,
    body: Json<v0::AdminObjectNoteEdit>,
) -> Result<EmptyResponse> {
    let user = flatten_authorized_user(&auth);
    if !user_has_permission(user, v0::AdminUserPermissionFlags::ObjectNotes) {
        return Err(create_error!(NotFound));
    }

    // Unfortunately due to how our system works, we can't limit users to editing objects they have permissions for (eg users, servers, etc.).
    // Hence it being tied to its own permission
    db.admin_note_update(AdminObjectNote::new(&object.id, &user.id, &body.content))
        .await?;
    Ok(EmptyResponse)
}
