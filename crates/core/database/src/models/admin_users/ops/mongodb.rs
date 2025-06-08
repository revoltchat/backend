use revolt_result::Result;

use crate::MongoDb;
use crate::{AdminUser, PartialAdminUser};

use super::AbstractAdminUsers;

static COL: &str = "admin_users";

#[async_trait]
impl AbstractAdminUsers for MongoDb {
    async fn admin_user_insert(&self, user: AdminUser) -> Result<()> {
        query!(self, insert_one, COL, user).map(|_| ())
    }

    async fn admin_user_update(&self, user_id: &str, partial: PartialAdminUser) -> Result<()> {
        query!(self, update_one_by_id, COL, user_id, partial, vec![], None).map(|_| ())
    }

    async fn admin_user_fetch(&self, user_id: &str) -> Result<AdminUser> {
        query!(self, find_one_by_id, COL, user_id)?.ok_or_else(|| create_error!(NotFound))
    }

    async fn admin_user_list(&self) -> Result<Vec<AdminUser>> {
        query!(self, find, COL, doc! {})
    }
}
