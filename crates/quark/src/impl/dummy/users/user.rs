use crate::models::user::{FieldsUser, PartialUser, RelationshipStatus, User};
use crate::{AbstractUser, Result};

use super::super::DummyDb;

#[async_trait]
impl AbstractUser for DummyDb {
    async fn fetch_user(&self, id: &str) -> Result<User> {
        Ok(User {
            id: id.into(),
            username: "username".into(),
            discriminator: "0000".into(),
            ..Default::default()
        })
    }

    async fn fetch_user_by_username(&self, username: &str, _discriminator: &str) -> Result<User> {
        self.fetch_user(username).await
    }

    async fn fetch_user_by_token(&self, token: &str) -> Result<User> {
        self.fetch_user(token).await
    }

    async fn insert_user(&self, user: &User) -> Result<()> {
        info!("Insert {:?}", user);
        Ok(())
    }

    async fn update_user(
        &self,
        id: &str,
        user: &PartialUser,
        remove: Vec<FieldsUser>,
    ) -> Result<()> {
        info!("Update {id} with {user:?} and remove {remove:?}");
        Ok(())
    }

    async fn delete_user(&self, id: &str) -> Result<()> {
        info!("Delete {id}");
        Ok(())
    }

    async fn fetch_users<'a>(&self, _id: &'a [String]) -> Result<Vec<User>> {
        Ok(vec![self.fetch_user("id").await.unwrap()])
    }

    async fn fetch_discriminators_in_use(&self, _username: &str) -> Result<Vec<String>> {
        Ok(vec![])
    }

    async fn fetch_mutual_user_ids(&self, _user_a: &str, _user_b: &str) -> Result<Vec<String>> {
        Ok(vec!["a".into()])
    }

    async fn fetch_mutual_channel_ids(&self, _user_a: &str, _user_b: &str) -> Result<Vec<String>> {
        Ok(vec!["b".into()])
    }

    async fn fetch_mutual_server_ids(&self, _user_a: &str, _user_b: &str) -> Result<Vec<String>> {
        Ok(vec!["c".into()])
    }

    async fn set_relationship(
        &self,
        user_id: &str,
        target_id: &str,
        relationship: &RelationshipStatus,
    ) -> Result<()> {
        info!("Set relationship from {user_id} to {target_id} as {relationship:?}");
        Ok(())
    }

    async fn pull_relationship(&self, user_id: &str, target_id: &str) -> Result<()> {
        info!("Removing relationship from {user_id} to {target_id}");
        Ok(())
    }
}
