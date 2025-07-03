mod mongodb;
mod reference;
use revolt_result::Result;

use crate::models::admin_users::models::{AdminUser, PartialAdminUser};

#[async_trait]
pub trait AbstractAdminUsers: Sync + Send {
    async fn admin_user_insert(&self, user: AdminUser) -> Result<()>;

    async fn admin_user_update(&self, user_id: &str, partial: PartialAdminUser) -> Result<()>;

    async fn admin_user_fetch(&self, user_id: &str) -> Result<AdminUser>;

    async fn admin_user_fetch_email(&self, email: &str) -> Result<AdminUser>;

    async fn admin_user_list(&self) -> Result<Vec<AdminUser>>;

    async fn admin_user_count(&self) -> Result<u16>;
}
