use revolt_database::{AdminAuthorization, AdminUser, AdminUserPermissionFlagsValue};
use revolt_models::v0::AdminUserPermissionFlags;

pub fn user_has_permission(user: &AdminUser, permission: AdminUserPermissionFlags) -> bool {
    let flags = AdminUserPermissionFlagsValue(user.permissions);

    flags.has(permission)
}

pub fn flatten_authorized_user<'a>(auth: &'a AdminAuthorization) -> &'a AdminUser {
    match auth {
        AdminAuthorization::AdminUser(admin_user) => &admin_user,
        AdminAuthorization::AdminMachine(admin_machine_token) => &admin_machine_token.on_behalf_of,
    }
}
