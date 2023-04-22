use crate::events::client::EventV1;
use crate::models::user::{
    Badges, FieldsUser, PartialUser, Presence, RelationshipStatus, User, UserHint,
};
use crate::permissions::defn::UserPerms;
use crate::permissions::r#impl::user::get_relationship;
use crate::{perms, Database, Error, Result};

use futures::try_join;
use impl_ops::impl_op_ex_commutative;
use revolt_presence::filter_online;
use std::ops;

impl_op_ex_commutative!(+ |a: &i32, b: &Badges| -> i32 { *a | *b as i32 });

impl User {
    /// Update user data
    pub async fn update<'a>(
        &mut self,
        db: &Database,
        partial: PartialUser,
        remove: Vec<FieldsUser>,
    ) -> Result<()> {
        for field in &remove {
            self.remove(field);
        }

        self.apply_options(partial.clone());

        db.update_user(&self.id, &partial, remove.clone()).await?;

        EventV1::UserUpdate {
            id: self.id.clone(),
            data: partial,
            clear: remove,
        }
        .p_user(self.id.clone(), db)
        .await;

        Ok(())
    }

    /// Remove a field from User object
    pub fn remove(&mut self, field: &FieldsUser) {
        match field {
            FieldsUser::Avatar => self.avatar = None,
            FieldsUser::StatusText => {
                if let Some(x) = self.status.as_mut() {
                    x.text = None;
                }
            }
            FieldsUser::StatusPresence => {
                if let Some(x) = self.status.as_mut() {
                    x.presence = None;
                }
            }
            FieldsUser::ProfileContent => {
                if let Some(x) = self.profile.as_mut() {
                    x.content = None;
                }
            }
            FieldsUser::ProfileBackground => {
                if let Some(x) = self.profile.as_mut() {
                    x.background = None;
                }
            }
        }
    }

    /// Mutate the user object to remove redundant information
    #[must_use]
    pub fn foreign(mut self) -> User {
        self.profile = None;
        self.relations = None;

        let mut badges = self.badges.unwrap_or(0);
        if let Ok(id) = ulid::Ulid::from_string(&self.id) {
            // Yes, this is hard-coded
            // No, I don't care + ratio
            if id.datetime().timestamp_millis() < 1629638578431 {
                badges = badges + Badges::EarlyAdopter;
            }
        }

        self.badges = Some(badges);

        if let Some(status) = &self.status {
            if let Some(presence) = &status.presence {
                if presence == &Presence::Invisible {
                    self.status = None;
                    self.online = Some(false);
                }
            }
        }

        self
    }

    /// Fetch foreign users by a list of IDs
    pub async fn fetch_foreign_users(db: &Database, user_ids: &[String]) -> Result<Vec<User>> {
        let online_ids = filter_online(user_ids).await;

        Ok(db
            .fetch_users(user_ids)
            .await?
            .into_iter()
            .map(|mut user| {
                user.online = Some(online_ids.contains(&user.id));
                user.foreign()
            })
            .collect::<Vec<User>>())
    }

    /// Mutate the user object to include relationship (if it does not already exist)
    #[must_use]
    pub fn with_relationship(self, perspective: &User) -> User {
        let mut user = self.foreign();

        if user.relationship.is_none() {
            user.relationship = Some(get_relationship(perspective, &user.id));
        }

        user
    }

    /// Mutate user object with given permission
    #[must_use]
    pub fn apply_permission(mut self, permission: &UserPerms) -> User {
        if !permission.get_view_profile() {
            self.status = None;
        }

        self
    }

    /// Helper function to apply relationship and permission
    #[must_use]
    pub fn with_perspective(self, perspective: &User, permission: &UserPerms) -> User {
        self.with_relationship(perspective)
            .apply_permission(permission)
    }

    /// Helper function to calculate perspective
    pub async fn with_auto_perspective(self, db: &Database, perspective: &User) -> User {
        let user = self.with_relationship(perspective);
        let permissions = perms(perspective).user(&user).calc_user(db).await;
        user.apply_permission(&permissions)
    }

    /// Check whether two users have a mutual connection
    ///
    /// This will check if user and user_b share a server or a group.
    pub async fn has_mutual_connection(&self, db: &Database, user_b: &str) -> Result<bool> {
        Ok(!db
            .fetch_mutual_server_ids(&self.id, user_b)
            .await?
            .is_empty()
            || !db
                .fetch_mutual_channel_ids(&self.id, user_b)
                .await?
                .is_empty())
    }

    /// Check if this user can acquire another server
    pub async fn can_acquire_server(&self, db: &Database) -> Result<bool> {
        // ! FIXME: hardcoded max server count
        Ok(db.fetch_server_count(&self.id).await? <= 100)
    }

    /// Sanitise and validate a username can be used
    pub async fn validate_username(db: &Database, username: String) -> Result<String> {
        // Trim surrounding spaces
        let username = username.trim().to_string();

        // Make sure username is still at least 3 characters
        if username.len() < 2 {
            return Err(Error::InvalidUsername);
        }

        // Copy the username for validation
        let username_lowercase = username.to_lowercase();

        // Ensure the username itself isn't blocked
        const BLOCKED_USERNAMES: &[&str] = &["admin", "revolt"];

        for username in BLOCKED_USERNAMES {
            if username_lowercase == *username {
                return Err(Error::InvalidUsername);
            }
        }

        // Ensure none of the following substrings show up in the username
        const BLOCKED_SUBSTRINGS: &[&str] = &["```"];

        for substr in BLOCKED_SUBSTRINGS {
            if username_lowercase.contains(substr) {
                return Err(Error::InvalidUsername);
            }
        }

        // Make sure the username isn't taken
        if db.is_username_taken(&username).await? {
            return Err(Error::UsernameTaken);
        }

        Ok(username)
    }

    /// Update a user's username
    pub async fn update_username(&mut self, db: &Database, username: String) -> Result<()> {
        self.update(
            db,
            PartialUser {
                username: Some(User::validate_username(db, username).await?),
                ..Default::default()
            },
            vec![],
        )
        .await
    }

    /// Apply a certain relationship between two users
    pub async fn apply_relationship(
        &self,
        db: &Database,
        target: &mut User,
        local: RelationshipStatus,
        remote: RelationshipStatus,
    ) -> Result<()> {
        if try_join!(
            db.set_relationship(&self.id, &target.id, &local),
            db.set_relationship(&target.id, &self.id, &remote)
        )
        .is_err()
        {
            return Err(Error::DatabaseError {
                operation: "update_one",
                with: "user",
            });
        }

        EventV1::UserRelationship {
            id: target.id.clone(),
            user: self.clone().with_relationship(target),
            status: remote,
        }
        .private(target.id.clone())
        .await;

        EventV1::UserRelationship {
            id: self.id.clone(),
            user: target.clone().with_relationship(self),
            status: local.clone(),
        }
        .private(self.id.clone())
        .await;

        target.relationship.replace(local);
        Ok(())
    }

    /// Add another user as a friend
    pub async fn add_friend(&self, db: &Database, target: &mut User) -> Result<()> {
        match get_relationship(self, &target.id) {
            RelationshipStatus::User => Err(Error::NoEffect),
            RelationshipStatus::Friend => Err(Error::AlreadyFriends),
            RelationshipStatus::Outgoing => Err(Error::AlreadySentRequest),
            RelationshipStatus::Blocked => Err(Error::Blocked),
            RelationshipStatus::BlockedOther => Err(Error::BlockedByOther),
            RelationshipStatus::Incoming => {
                self.apply_relationship(
                    db,
                    target,
                    RelationshipStatus::Friend,
                    RelationshipStatus::Friend,
                )
                .await
            }
            RelationshipStatus::None => {
                self.apply_relationship(
                    db,
                    target,
                    RelationshipStatus::Outgoing,
                    RelationshipStatus::Incoming,
                )
                .await
            }
        }
    }

    /// Remove another user as a friend
    pub async fn remove_friend(&self, db: &Database, target: &mut User) -> Result<()> {
        match get_relationship(self, &target.id) {
            RelationshipStatus::Friend
            | RelationshipStatus::Outgoing
            | RelationshipStatus::Incoming => {
                self.apply_relationship(
                    db,
                    target,
                    RelationshipStatus::None,
                    RelationshipStatus::None,
                )
                .await
            }
            _ => Err(Error::NoEffect),
        }
    }

    /// Block another user
    pub async fn block_user(&self, db: &Database, target: &mut User) -> Result<()> {
        match get_relationship(self, &target.id) {
            RelationshipStatus::User | RelationshipStatus::Blocked => Err(Error::NoEffect),
            RelationshipStatus::BlockedOther => {
                self.apply_relationship(
                    db,
                    target,
                    RelationshipStatus::Blocked,
                    RelationshipStatus::Blocked,
                )
                .await
            }
            RelationshipStatus::None
            | RelationshipStatus::Friend
            | RelationshipStatus::Incoming
            | RelationshipStatus::Outgoing => {
                self.apply_relationship(
                    db,
                    target,
                    RelationshipStatus::Blocked,
                    RelationshipStatus::BlockedOther,
                )
                .await
            }
        }
    }

    /// Unblock another user
    pub async fn unblock_user(&self, db: &Database, target: &mut User) -> Result<()> {
        match get_relationship(self, &target.id) {
            RelationshipStatus::Blocked => match get_relationship(target, &self.id) {
                RelationshipStatus::Blocked => {
                    self.apply_relationship(
                        db,
                        target,
                        RelationshipStatus::BlockedOther,
                        RelationshipStatus::Blocked,
                    )
                    .await
                }
                RelationshipStatus::BlockedOther => {
                    self.apply_relationship(
                        db,
                        target,
                        RelationshipStatus::None,
                        RelationshipStatus::None,
                    )
                    .await
                }
                _ => Err(Error::InternalError),
            },
            _ => Err(Error::NoEffect),
        }
    }

    /// Check whether this user has another user blocked
    pub fn has_blocked(&self, user: &str) -> bool {
        matches!(
            get_relationship(self, user),
            RelationshipStatus::Blocked | RelationshipStatus::BlockedOther
        )
    }

    /// Mark as deleted
    pub async fn mark_deleted(&mut self, db: &Database) -> Result<()> {
        self.update(
            db,
            PartialUser {
                username: Some(format!("Deleted User {}", self.id)),
                flags: Some(2),
                ..Default::default()
            },
            vec![
                FieldsUser::Avatar,
                FieldsUser::StatusText,
                FieldsUser::StatusPresence,
                FieldsUser::ProfileContent,
                FieldsUser::ProfileBackground,
            ],
        )
        .await
    }

    /// Find a user from a given token and hint
    #[async_recursion]
    pub async fn from_token(db: &Database, token: &str, hint: UserHint) -> Result<User> {
        match hint {
            UserHint::Bot => db.fetch_user(&db.fetch_bot_by_token(token).await?.id).await,
            UserHint::User => db.fetch_user_by_token(token).await,
            UserHint::Any => {
                if let Ok(user) = User::from_token(db, token, UserHint::User).await {
                    Ok(user)
                } else {
                    User::from_token(db, token, UserHint::Bot).await
                }
            }
        }
    }
}
