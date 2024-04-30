use crate::{
    models::{user::RelationshipStatus, Channel, Member, Server, User},
    util::value::Value,
    Database, Error, Override, Permission, Result,
};

pub mod defn;
pub mod r#impl;

pub use r#impl::user::get_relationship;

/// Permissions calculator
#[derive(Clone)]
pub struct PermissionCalculator<'a> {
    perspective: &'a User,

    pub user: Value<'a, User>,
    pub channel: Value<'a, Channel>,
    pub server: Value<'a, Server>,
    pub member: Value<'a, Member>,

    flag_known_relationship: Option<&'a RelationshipStatus>,
    flag_has_mutual_connection: bool,

    cached_user_permission: Option<u32>,
    cached_permission: Option<u64>,
}

impl<'a> PermissionCalculator<'a> {
    /// Create a new permission calculator
    pub fn new(perspective: &'a User) -> PermissionCalculator {
        PermissionCalculator {
            perspective,

            user: Value::None,
            channel: Value::None,
            server: Value::None,
            member: Value::None,

            flag_known_relationship: None,
            flag_has_mutual_connection: false,

            cached_user_permission: None,
            cached_permission: None,
        }
    }

    /// Use user by ref
    pub fn user(self, user: &'a User) -> PermissionCalculator {
        PermissionCalculator {
            user: Value::Ref(user),
            ..self
        }
    }

    /// Use channel by ref
    pub fn channel(self, channel: &'a Channel) -> PermissionCalculator {
        PermissionCalculator {
            channel: Value::Ref(channel),
            ..self
        }
    }

    /// Use server by ref
    pub fn server(self, server: &'a Server) -> PermissionCalculator {
        PermissionCalculator {
            server: Value::Ref(server),
            ..self
        }
    }

    /// Use member by ref
    pub fn member(self, member: &'a Member) -> PermissionCalculator {
        PermissionCalculator {
            member: Value::Ref(member),
            ..self
        }
    }

    /// Use existing relationship by ref
    pub fn with_relationship(self, relationship: &'a RelationshipStatus) -> PermissionCalculator {
        PermissionCalculator {
            flag_known_relationship: Some(relationship),
            ..self
        }
    }

    /// Check whether the calculated permission contains a given value
    pub async fn has_permission_value(&mut self, db: &Database, value: u64) -> Result<bool> {
        let perms = if let Some(perms) = self.cached_permission {
            perms
        } else {
            self.calc(db).await?.0[0]
        };

        Ok((value) & perms == (value))
    }

    /// Check whether we have a given permission
    pub async fn has_permission(&mut self, db: &Database, permission: Permission) -> Result<bool> {
        self.has_permission_value(db, permission as u64).await
    }

    /// Check whether we have a given permission, otherwise throw an error
    pub async fn throw_permission_value(&mut self, db: &Database, value: u64) -> Result<()> {
        if self.has_permission_value(db, value).await? {
            Ok(())
        } else {
            Err(Error::CannotGiveMissingPermissions)
        }
    }

    /// Check whether we have a given permission, otherwise throw an error
    pub async fn throw_permission(&mut self, db: &Database, permission: Permission) -> Result<()> {
        if self.has_permission(db, permission).await? {
            Ok(())
        } else {
            Error::from_permission(permission)
        }
    }

    /// Throw an error if we cannot grant permissions on either allows or denies
    /// going from the previous given value to the next given value.
    ///
    /// We need to check any:
    /// - allows added (permissions now granted)
    /// - denies removed (permissions now neutral or granted)
    pub async fn throw_permission_override<C>(
        &mut self,
        db: &Database,
        current_value: C,
        next_value: Override,
    ) -> Result<()>
    where
        C: Into<Option<Override>>,
    {
        let current_value = current_value.into();

        if let Some(current_value) = current_value {
            self.throw_permission_value(db, !current_value.allows() & next_value.allows())
                .await?;

            self.throw_permission_value(db, current_value.denies() & !next_value.denies())
                .await
        } else {
            self.throw_permission_value(db, next_value.allows()).await
        }
    }

    /// Check whether we has the ViewChannel and another given permission, otherwise throw an error
    pub async fn throw_permission_and_view_channel(
        &mut self,
        db: &Database,
        permission: Permission,
    ) -> Result<()> {
        self.throw_permission(db, Permission::ViewChannel).await?;
        self.throw_permission(db, permission).await
    }

    /// Get the known member's current ranking
    pub fn get_member_rank(&self) -> Option<i64> {
        self.member
            .get()
            .map(|member| member.get_ranking(self.server.get().unwrap()))
    }
}

/// Short-hand for creating a permission calculator
pub fn perms(perspective: &'_ User) -> PermissionCalculator<'_> {
    PermissionCalculator::new(perspective)
}
