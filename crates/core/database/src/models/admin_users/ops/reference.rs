use revolt_result::Result;

use crate::ReferenceDb;
use crate::{AdminUser, PartialAdminUser};

use super::AbstractAdminUsers;

#[async_trait]
impl AbstractAdminUsers for ReferenceDb {
    async fn admin_user_insert(&self, user: AdminUser) -> Result<()> {
        let mut admin_users = self.admin_users.lock().await;
        if admin_users.contains_key(&user.id) {
            Err(create_database_error!("insert", "admin_users"))
        } else {
            admin_users.insert(user.id.to_string(), user.clone());
            Ok(())
        }
    }

    async fn admin_user_update(&self, user_id: &str, partial: PartialAdminUser) -> Result<()> {
        let mut admin_users = self.admin_users.lock().await;
        if let Some(existing_user) = admin_users.get_mut(user_id) {
            existing_user.apply_options(partial);
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }

    async fn admin_user_fetch(&self, user_id: &str) -> Result<AdminUser> {
        let admin_users = self.admin_users.lock().await;
        if let Some(user) = admin_users.get(user_id) {
            Ok(user.clone())
        } else {
            Err(create_error!(NotFound))
        }
    }

    async fn admin_user_fetch_email(&self, email: &str) -> Result<AdminUser> {
        let admin_users = self.admin_users.lock().await;
        if let Some((_, user)) = admin_users.iter().filter(|(_, p)| p.email == email).next() {
            Ok(user.clone())
        } else {
            Err(create_error!(NotFound))
        }
    }

    async fn admin_user_list(&self) -> Result<Vec<AdminUser>> {
        let admin_users = self.admin_users.lock().await;
        Ok(admin_users.iter().map(|(_, u)| u).cloned().collect())
    }
}
