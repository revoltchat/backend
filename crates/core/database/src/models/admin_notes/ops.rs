mod mongodb;
mod reference;
use revolt_result::Result;

use crate::models::admin_notes::AdminObjectNote;

#[async_trait]
pub trait AbstractAdminNotes: Sync + Send {
    async fn admin_note_update(&self, note: AdminObjectNote) -> Result<()>;

    async fn admin_note_fetch(&self, target_id: &str) -> Result<AdminObjectNote>;
}
