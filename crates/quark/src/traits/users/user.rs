use crate::models::user::{FieldsUser, PartialUser, RelationshipStatus, User};
use crate::Result;

#[async_trait]
pub trait AbstractUser: Sync + Send {
    /// Fetch a user from the database
    async fn fetch_user(&self, id: &str) -> Result<User>;

    /// Fetch a user from the database by their username
    async fn fetch_user_by_username(&self, username: &str) -> Result<User>;

    /// Fetch a user from the database by their session token
    async fn fetch_user_by_token(&self, token: &str) -> Result<User>;

    /// Insert a new user into the database
    async fn insert_user(&self, user: &User) -> Result<()>;

    /// Update a user by their id given some data
    async fn update_user(
        &self,
        id: &str,
        user: &PartialUser,
        remove: Vec<FieldsUser>,
    ) -> Result<()>;

    /// Delete a user by their id
    async fn delete_user(&self, id: &str) -> Result<()>;

    /// Fetch multiple users by their ids
    async fn fetch_users<'a>(&self, ids: &'a [String]) -> Result<Vec<User>>;

    /// Check whether a username is already in use by another user
    async fn is_username_taken(&self, username: &str) -> Result<bool>;

    /// Fetch ids of users that both users are friends with
    async fn fetch_mutual_user_ids(&self, user_a: &str, user_b: &str) -> Result<Vec<String>>;

    /// Fetch ids of channels that both users are in
    async fn fetch_mutual_channel_ids(&self, user_a: &str, user_b: &str) -> Result<Vec<String>>;

    /// Fetch ids of servers that both users share
    async fn fetch_mutual_server_ids(&self, user_a: &str, user_b: &str) -> Result<Vec<String>>;

    /// Set relationship with another user
    ///
    /// This should use pull_relationship if relationship is None.
    async fn set_relationship(
        &self,
        user_id: &str,
        target_id: &str,
        relationship: &RelationshipStatus,
    ) -> Result<()>;

    /// Remove relationship with another user
    async fn pull_relationship(&self, user_id: &str, target_id: &str) -> Result<()>;
}
