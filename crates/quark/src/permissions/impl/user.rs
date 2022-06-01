use crate::{
    models::{user::RelationshipStatus, User},
    permissions::PermissionCalculator,
    UserPermission, UserPermissions, UserPerms,
};

impl PermissionCalculator<'_> {
    /// Calculate the permissions from our perspective to the given user
    ///
    /// How the permission is calculated:
    /// 1. Are we the target?
    ///     - If so: return maximum permissions
    /// 2. Do we have a relationship with the target?
    ///     - If we are friends: return maximum permissions
    ///     - If either user blocked each other: return only `Access`
    ///     - If incoming / outgoing request: add `Access` to the list
    /// 3. Determine whether there is a mutual connection:
    ///     1. Check if the "mutual connection" flag is set.
    ///     2. Check if we share any servers with the target.
    ///     3. Check if we share any DMs or groups with the target.
    /// 4. Do we have a mutual connection with the target?
    ///     - If so: return `Access` + `ViewProfile`
    /// 5. Return no permissions
    pub async fn calc_user(&mut self, db: &crate::Database) -> UserPerms {
        if self.user.has() {
            let v = calculate_permission(self, db).await;
            self.cached_user_permission = Some(v);
            UserPermissions([v])
        } else {
            panic!("Expected `PermissionCalculator.user` to exist.")
        }
    }
}

/// Find the relationship between two users
pub fn get_relationship(a: &User, b: &str) -> RelationshipStatus {
    if a.id == b {
        return RelationshipStatus::User;
    }

    if let Some(relations) = &a.relations {
        if let Some(relationship) = relations.iter().find(|x| x.id == b) {
            return relationship.status.clone();
        }
    }

    RelationshipStatus::None
}

/// Internal helper function for calculating permission
async fn calculate_permission(data: &mut PermissionCalculator<'_>, db: &crate::Database) -> u32 {
    let user = data.user.get().unwrap();

    if data.perspective.id == user.id {
        return u32::MAX;
    }

    let relationship = data.flag_known_relationship.cloned().unwrap_or_else(|| {
        user.relationship
            .as_ref()
            .cloned()
            .unwrap_or_else(|| get_relationship(data.perspective, &user.id))
    });

    let mut permissions: u32 = 0;
    match relationship {
        RelationshipStatus::Friend => return u32::MAX,
        RelationshipStatus::Blocked | RelationshipStatus::BlockedOther => {
            return UserPermission::Access as u32
        }
        RelationshipStatus::Incoming | RelationshipStatus::Outgoing => {
            permissions = UserPermission::Access as u32;
        }
        _ => {}
    }

    // ! FIXME: add boolean switch for permission for users to globally message a user
    // maybe an enum?
    // PrivacyLevel { Private, Friends, Mutual, Public, Global }

    // ! FIXME: add boolean switch for permission for users to mutually DM a user

    if data.flag_has_mutual_connection
        || data
            .perspective
            .has_mutual_connection(db, &user.id)
            .await
            .unwrap_or(false)
    {
        permissions = UserPermission::Access + UserPermission::ViewProfile;

        if user.bot.is_some() || data.perspective.bot.is_some() {
            permissions += UserPermission::SendMessage as u32;
        }

        return permissions;
    }

    permissions
}
